use crate::controllers::podcast_episode_controller::PodcastEpisodeWithHistory;
use crate::history::map_episode_to_dto;
use crate::podcast_episode_dto::PodcastEpisodeDto;
use crate::usecases::podcast_episode::PodcastEpisodeUseCase as PodcastEpisodeService;
use crate::usecases::watchtime::WatchtimeUseCase as WatchtimeService;
use chrono::Utc;
use common_infrastructure::error::{CustomError, CustomErrorInner, ErrorSeverity};
use podfetch_domain::episode_triage::{EpisodeTriage, EpisodeTriageRepository, TriageStatus};
use podfetch_domain::user::User;
use podfetch_persistence::adapters::EpisodeTriageRepositoryImpl;
use podfetch_persistence::db::database;
use podfetch_persistence::podcast_episode::PodcastEpisodeEntity as PodcastEpisode;
use std::sync::Arc;
use uuid::Uuid;

/// Page size cap when listing the inbox / archive.
pub const DEFAULT_PAGE_SIZE: i64 = 30;
/// Upper bound on how many episodes a single "clear inbox" call will dismiss.
/// Generous enough for any realistic inbox while bounding worst-case work.
const CLEAR_INBOX_LIMIT: i64 = 10_000;

#[derive(Clone)]
pub struct EpisodeTriageService {
    repository: Arc<dyn EpisodeTriageRepository<Error = CustomError>>,
}

impl EpisodeTriageService {
    pub fn new(repository: Arc<dyn EpisodeTriageRepository<Error = CustomError>>) -> Self {
        Self { repository }
    }

    pub fn default_service() -> Self {
        Self::new(Arc::new(EpisodeTriageRepositoryImpl::new(database())))
    }

    /// Build a `PodcastEpisodeWithHistory` DTO for a single episode, attaching
    /// the requesting user's listen history so the UI can render progress.
    fn to_item(episode: PodcastEpisode, user: &User, server_url: &str) -> PodcastEpisodeWithHistory {
        let history = WatchtimeService::get_watchtime(&episode.episode_id, &user.username)
            .ok()
            .flatten();

        PodcastEpisodeWithHistory {
            podcast_episode: PodcastEpisodeDto::from_episode_with_user(
                episode,
                Some(user.clone()),
                None,
                server_url,
            ),
            podcast_history_item: history.map(Into::into).as_ref().map(map_episode_to_dto),
        }
    }

    /// Inbox: not-yet-downloaded, non-deleted episodes the user has not triaged.
    pub fn get_inbox(
        &self,
        user: &User,
        last_date: Option<String>,
        limit: i64,
        server_url: &str,
    ) -> Result<Vec<PodcastEpisodeWithHistory>, CustomError> {
        let triaged = self.repository.list_triaged_episode_ids(user.id)?;
        let episodes =
            PodcastEpisodeService::get_inbox_episodes(&triaged, last_date.as_deref(), limit)?;
        Ok(episodes
            .into_iter()
            .map(|episode| Self::to_item(episode, user, server_url))
            .collect())
    }

    /// Waiting list: episodes the user picked (`queued`), newest first. Includes
    /// not-yet-finished downloads so the user sees what they selected.
    pub fn get_waiting_list(
        &self,
        user: &User,
        server_url: &str,
    ) -> Result<Vec<PodcastEpisodeWithHistory>, CustomError> {
        let ids = self
            .repository
            .list_episode_ids_by_status(user.id, TriageStatus::Queued)?;

        let mut episodes: Vec<PodcastEpisode> = ids
            .into_iter()
            .filter_map(|id| {
                PodcastEpisodeService::get_podcast_episode_by_internal_id(id)
                    .ok()
                    .flatten()
            })
            .collect();
        episodes.sort_by(|a, b| b.date_of_recording.cmp(&a.date_of_recording));

        Ok(episodes
            .into_iter()
            .map(|episode| Self::to_item(episode, user, server_url))
            .collect())
    }

    /// Archive: every downloaded, non-deleted episode, newest first.
    pub fn get_archive(
        &self,
        user: &User,
        last_date: Option<String>,
        limit: i64,
        server_url: &str,
    ) -> Result<Vec<PodcastEpisodeWithHistory>, CustomError> {
        let episodes =
            PodcastEpisodeService::get_downloaded_episodes_paginated(last_date.as_deref(), limit)?;
        Ok(episodes
            .into_iter()
            .map(|episode| Self::to_item(episode, user, server_url))
            .collect())
    }

    /// Record a triage decision for an episode. Triggering the actual download
    /// for `queued` episodes is the controller's responsibility.
    pub fn set_status(
        &self,
        user_id: Uuid,
        episode_id: Uuid,
        status: TriageStatus,
    ) -> Result<(), CustomError> {
        self.repository.upsert(EpisodeTriage {
            user_id,
            episode_id,
            status,
            updated_at: Utc::now().naive_utc(),
        })
    }

    /// Dismiss every episode currently in the user's inbox.
    pub fn clear_inbox(&self, user: &User) -> Result<usize, CustomError> {
        let triaged = self.repository.list_triaged_episode_ids(user.id)?;
        let inbox = PodcastEpisodeService::get_inbox_episodes(&triaged, None, CLEAR_INBOX_LIMIT)?;
        let now = Utc::now().naive_utc();

        let mut dismissed = 0usize;
        for episode in inbox {
            let episode_id = Uuid::parse_str(&episode.id).map_err(|_| -> CustomError {
                CustomErrorInner::Conflict(
                    format!("stored episode id '{}' is not a uuid", episode.id),
                    ErrorSeverity::Error,
                )
                .into()
            })?;
            self.repository.upsert(EpisodeTriage {
                user_id: user.id,
                episode_id,
                status: TriageStatus::Dismissed,
                updated_at: now,
            })?;
            dismissed += 1;
        }
        Ok(dismissed)
    }

    /// Remove all triage rows referencing an episode (cleanup on episode/podcast
    /// deletion).
    pub fn delete_triage_for_episode(&self, episode_id: Uuid) -> Result<(), CustomError> {
        self.repository.delete_by_episode_id(episode_id)?;
        Ok(())
    }
}
