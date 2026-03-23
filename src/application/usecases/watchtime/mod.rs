use crate::adapters::api::models::podcast_episode_dto::PodcastEpisodeDto;
use crate::adapters::persistence::dbconfig::db::database;
use crate::application::services::listening_event::service::ListeningEventService;
use crate::adapters::api::mappers::episode::map_episode_to_dto;
use crate::models::episode::Episode;
use crate::utils::error::CustomError;
use crate::adapters::api::mappers::podcast::map_podcast_to_dto;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcasts::Podcast;
use chrono::{NaiveDateTime, Utc};
use podfetch_domain::episode::{EpisodeRepository, NewEpisode};
use podfetch_domain::favorite_podcast_episode::FavoritePodcastEpisode;
use podfetch_domain::listening_event::NewListeningEvent;
use podfetch_domain::user::User;
use podfetch_web::history::EpisodeDto;
use podfetch_web::podcast::PodcastDto;
use podfetch_web::watchtime::{
    PodcastWatchedEpisodeModelWithPodcastEpisode, PodcastWatchedPostModel,
    WatchtimeApplicationService,
};
use podfetch_persistence::episode::DieselEpisodeRepository;

use crate::constants::inner_constants::DEFAULT_DEVICE;
use crate::utils::error::ErrorSeverity::Warning;
use crate::utils::error::CustomErrorInner;

#[derive(Clone, Default)]
pub struct WatchtimeUseCase;

const LISTENING_DELTA_GRACE_SECONDS: i64 = 15;

impl WatchtimeUseCase {
    pub fn new() -> Self {
        Self
    }

    fn repo() -> DieselEpisodeRepository {
        DieselEpisodeRepository::new(database())
    }

    pub fn insert_episode(
        episode: podfetch_domain::episode::Episode,
    ) -> Result<Episode, CustomError> {
        Self::repo()
            .insert_episode(&episode)
            .map(Into::into)
            .map_err(Into::into)
    }

    pub fn get_actions_by_username(
        username: &str,
        since_date: Option<NaiveDateTime>,
        opt_device: Option<String>,
        _opt_aggregate: Option<String>,
        opt_podcast: Option<String>,
    ) -> Result<Vec<Episode>, CustomError> {
        Self::repo()
            .find_actions_by_username(
                username,
                since_date,
                opt_device.as_deref(),
                opt_podcast.as_deref(),
                Some(DEFAULT_DEVICE),
            )
            .map(|rows| rows.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    pub fn get_watchtime(episode_id: &str, username: &str) -> Result<Option<Episode>, CustomError> {
        Self::repo()
            .find_watchtime(episode_id, username)
            .map(|episode| episode.map(Into::into))
            .map_err(Into::into)
    }

    pub fn get_last_watched_episodes(
        user: &User,
    ) -> Result<Vec<(PodcastEpisode, Episode, Podcast)>, CustomError> {
        Self::repo()
            .find_last_watched_episodes(&user.username)
            .map(|items| {
                items.into_iter()
                    .map(|item| {
                        (
                            item.podcast_episode.into(),
                            item.episode_action.into(),
                            item.podcast.into(),
                        )
                    })
                    .collect()
            })
            .map_err(Into::into)
    }

    pub fn delete_by_username_and_episode(username: &str) -> Result<(), CustomError> {
        ListeningEventService::default_service().delete_by_username(username)?;
        Self::repo().delete_by_username(username).map_err(Into::into)
    }

    pub fn delete_watchtime(podcast_id: i32) -> Result<(), CustomError> {
        ListeningEventService::default_service().delete_by_podcast_id(podcast_id)?;
        Self::repo()
            .delete_by_podcast_id(podcast_id)
            .map_err(Into::into)
    }

    pub fn delete_by_username(username: &str) -> Result<(), CustomError> {
        ListeningEventService::default_service().delete_by_username(username)?;
        Self::repo().delete_by_username(username).map_err(Into::into)
    }

    pub fn log_watchtime(
        podcast_episode_id: &str,
        watch_time: i32,
        username: String,
    ) -> Result<(), CustomError> {
        let found_episode = crate::application::usecases::podcast_episode::PodcastEpisodeUseCase::get_podcast_episode_by_id(
            podcast_episode_id,
        )?;

        let Some(found_episode) = found_episode else {
            return Err(CustomErrorInner::NotFound(Warning).into());
        };

        let podcast = crate::application::services::podcast::service::PodcastService::get_podcast(found_episode.podcast_id)?;

        let now = Utc::now().naive_utc();

        match Self::get_watchlog_by_device_and_episode(
            &username,
            &found_episode.guid,
            DEFAULT_DEVICE,
        )? {
            Some(episode) => {
                let listened_delta_seconds = Self::calculate_listened_delta(
                    episode.position,
                    Some(episode.timestamp),
                    watch_time,
                    now,
                );
                if listened_delta_seconds > 0 {
                    ListeningEventService::create_event(NewListeningEvent {
                        username: username.clone(),
                        device: DEFAULT_DEVICE.to_string(),
                        podcast_episode_id: found_episode.episode_id.clone(),
                        podcast_id: found_episode.podcast_id,
                        podcast_episode_db_id: found_episode.id,
                        delta_seconds: listened_delta_seconds,
                        start_position: watch_time.saturating_sub(listened_delta_seconds),
                        end_position: watch_time,
                        listened_at: now,
                    })?;
                }

                Self::repo()
                    .update_position(episode.id, watch_time, now)
                    .map_err(CustomError::from)?;
            }
            None => {
                Self::repo()
                    .create(NewEpisode {
                        username,
                        device: DEFAULT_DEVICE.to_string(),
                        podcast: podcast.rssfeed,
                        episode: found_episode.url,
                        timestamp: now,
                        guid: Some(found_episode.guid),
                        action: "play".to_string(),
                        started: Some(watch_time),
                        position: Some(watch_time),
                        total: Some(found_episode.total_time),
                    })
                    .map_err(CustomError::from)?;
            }
        }

        Ok(())
    }

    pub fn get_watchlog_by_device_and_episode(
        username: &str,
        episode_guid: &str,
        device_id: &str,
    ) -> Result<Option<Episode>, CustomError> {
        Self::repo()
            .find_by_username_device_guid(username, device_id, episode_guid)
            .map(|episode| episode.map(Into::into))
            .map_err(Into::into)
    }

    fn calculate_listened_delta(
        previous_position: Option<i32>,
        previous_timestamp: Option<NaiveDateTime>,
        current_position: i32,
        now: NaiveDateTime,
    ) -> i32 {
        let Some(previous_position) = previous_position else {
            return 0;
        };

        if current_position <= previous_position {
            return 0;
        }

        let raw_delta = current_position.saturating_sub(previous_position);
        let Some(previous_timestamp) = previous_timestamp else {
            return raw_delta;
        };

        let elapsed = now
            .signed_duration_since(previous_timestamp)
            .num_seconds()
            .max(0);
        let max_allowed = elapsed.saturating_add(LISTENING_DELTA_GRACE_SECONDS);
        raw_delta.min(max_allowed.min(i64::from(i32::MAX)) as i32)
    }
}

impl WatchtimeApplicationService for WatchtimeUseCase {
    type Error = CustomError;
    type EpisodeDto = EpisodeDto;
    type LastWatchedItem =
        PodcastWatchedEpisodeModelWithPodcastEpisode<PodcastEpisodeDto, PodcastDto, EpisodeDto>;

    fn log_watchtime(
        &self,
        username: String,
        request: PodcastWatchedPostModel,
    ) -> Result<(), Self::Error> {
        Self::log_watchtime(&request.podcast_episode_id, request.time, username)
    }

    fn get_last_watched(&self, username: &str) -> Result<Vec<Self::LastWatchedItem>, Self::Error> {
        let user = User::new(
            0,
            username.to_string(),
            "user",
            None::<String>,
            chrono::Utc::now().naive_utc(),
            true,
        );
        Self::get_last_watched_episodes(&user)
        .map(|items| {
            items.into_iter()
                .map(|(podcast_episode, episode, podcast)| {
                    PodcastWatchedEpisodeModelWithPodcastEpisode {
                        podcast_episode: (
                            podcast_episode,
                            Some(user.clone()),
                            None::<FavoritePodcastEpisode>,
                        )
                            .into(),
                        podcast: map_podcast_to_dto(podcast.into()),
                        episode: map_episode_to_dto(&episode.into()),
                    }
                })
                .collect()
        })
    }

    fn get_watchtime(
        &self,
        episode_id: &str,
        username: &str,
    ) -> Result<Option<Self::EpisodeDto>, Self::Error> {
        Self::get_watchtime(episode_id, username)
            .map(|episode| episode.map(Into::into).as_ref().map(map_episode_to_dto))
    }
}

#[cfg(test)]
mod tests {
    use super::WatchtimeUseCase;
    use chrono::{NaiveDate, NaiveDateTime};

    fn dt(hour: u32, minute: u32, second: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 2, 27)
            .unwrap()
            .and_hms_opt(hour, minute, second)
            .unwrap()
    }

    #[test]
    fn calculate_listened_delta_returns_zero_without_previous_position() {
        let delta = WatchtimeUseCase::calculate_listened_delta(None, None, 10, dt(10, 0, 0));
        assert_eq!(delta, 0);
    }

    #[test]
    fn calculate_listened_delta_returns_zero_for_seek_back() {
        let delta = WatchtimeUseCase::calculate_listened_delta(
            Some(30),
            Some(dt(10, 0, 0)),
            20,
            dt(10, 0, 4),
        );
        assert_eq!(delta, 0);
    }

    #[test]
    fn calculate_listened_delta_caps_large_forward_seek() {
        let delta = WatchtimeUseCase::calculate_listened_delta(
            Some(10),
            Some(dt(10, 0, 0)),
            400,
            dt(10, 0, 5),
        );
        assert_eq!(delta, 20);
    }

    #[test]
    fn calculate_listened_delta_keeps_regular_progress() {
        let delta = WatchtimeUseCase::calculate_listened_delta(
            Some(100),
            Some(dt(10, 0, 0)),
            108,
            dt(10, 0, 3),
        );
        assert_eq!(delta, 8);
    }
}

