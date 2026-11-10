use crate::history::EpisodeDto;
use crate::history::map_episode_to_dto;
use crate::podcast::PodcastDto;
use crate::podcast::map_podcast_to_dto;
use crate::podcast_episode_dto::PodcastEpisodeDto;
use crate::services::listening_event::service::ListeningEventService;
use crate::watchtime::{
    PodcastWatchedEpisodeModelWithPodcastEpisode, PodcastWatchedPostModel,
    WatchtimeApplicationService,
};
use chrono::{NaiveDateTime, Utc};
use common_infrastructure::error::CustomError;
use podfetch_domain::episode::{EpisodeRepository, NewEpisode};
use podfetch_domain::favorite_podcast_episode::FavoritePodcastEpisode;
use podfetch_domain::listening_event::NewListeningEvent;
use podfetch_domain::user::User;
use podfetch_domain::user_admin::UserAdminRepository;
use podfetch_persistence::db::database;
use podfetch_persistence::episode::DieselEpisodeRepository;
use podfetch_persistence::episode::EpisodeEntity as Episode;
use podfetch_persistence::podcast::PodcastEntity as Podcast;
use podfetch_persistence::podcast_episode::PodcastEpisodeEntity as PodcastEpisode;
use podfetch_persistence::user_admin::DieselUserAdminRepository;

use common_infrastructure::error::CustomErrorInner;
use common_infrastructure::error::ErrorSeverity::Warning;
use common_infrastructure::runtime::DEFAULT_DEVICE;
use uuid::Uuid;

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

    fn get_user_id_by_username(username: &str) -> Result<Uuid, CustomError> {
        DieselUserAdminRepository::new(database())
            .find_by_username(username)?
            .map(|user| user.id)
            .ok_or_else(|| CustomErrorInner::NotFound(Warning).into())
    }

    /// Parse a stored podfetch id (entity rows carry the canonical UUID as a
    /// `String`) into a `Uuid`.
    fn parse_id(id: &str) -> Result<Uuid, CustomError> {
        Uuid::parse_str(id).map_err(|_| CustomErrorInner::NotFound(Warning).into())
    }

    pub fn insert_episode(
        episode: podfetch_domain::episode::Episode,
    ) -> Result<Episode, CustomError> {
        Self::repo()
            .insert_episode(&episode)
            .map(Into::into)
            .map_err(Into::into)
    }

    /// Insert or update an episode action with GUID-based fallback matching.
    ///
    /// When a client (e.g. AntennaPod) uses a local RSS feed, the episode/podcast URLs
    /// won't match the original URLs stored by PodFetch. If a GUID is present, we first
    /// check for an existing action with the same GUID and update it instead of creating
    /// a duplicate entry.
    ///
    /// Every new `play` action is additionally recorded as a listening event so
    /// that listening done in GPodder clients shows up on the statistics page.
    pub fn upsert_episode_by_guid(
        episode: podfetch_domain::episode::Episode,
    ) -> Result<Episode, CustomError> {
        let repo = Self::repo();

        if let Some(ref guid) = episode.guid
            && let Some(existing) = repo
                .find_by_username_and_guid(&episode.username, guid)
                .map_err(CustomError::from)?
        {
            let is_duplicate =
                existing.timestamp == episode.timestamp && existing.position == episode.position;
            if !is_duplicate {
                Self::record_gpodder_listening_event(&episode, Some(&existing));
            }
            if let Some(position) = episode.position {
                repo.update_position(existing.id, position, episode.timestamp)
                    .map_err(CustomError::from)?;
            }
            return repo
                .find_by_username_and_guid(&episode.username, guid)
                .map(|opt| opt.unwrap().into())
                .map_err(Into::into);
        }

        if let Some(existing) = repo
            .find_duplicate_action(&episode)
            .map_err(CustomError::from)?
        {
            return Ok(existing.into());
        }

        let inserted = repo.insert_episode(&episode).map_err(CustomError::from)?;
        Self::record_gpodder_listening_event(&episode, None);
        Ok(inserted.into())
    }

    /// Create a listening event for a `play` action uploaded by a GPodder client.
    ///
    /// Failures are logged and swallowed: a missing statistics entry must never
    /// make the client's sync request fail.
    fn record_gpodder_listening_event(
        action: &podfetch_domain::episode::Episode,
        previous: Option<&podfetch_domain::episode::Episode>,
    ) {
        if let Err(err) = Self::try_record_gpodder_listening_event(action, previous) {
            tracing::warn!(
                "Could not record listening event for GPodder action of {}: {err:?}",
                action.username
            );
        }
    }

    fn try_record_gpodder_listening_event(
        action: &podfetch_domain::episode::Episode,
        previous: Option<&podfetch_domain::episode::Episode>,
    ) -> Result<(), CustomError> {
        if !action.action.eq_ignore_ascii_case("play") {
            return Ok(());
        }
        let Some(position) = action.position else {
            return Ok(());
        };
        let delta = Self::calculate_gpodder_delta(
            action.started,
            previous.and_then(|p| p.position),
            position,
            action.total,
        );
        if delta <= 0 {
            return Ok(());
        }

        let Some(podcast_episode) = Self::find_podcast_episode_for_action(action)? else {
            return Ok(());
        };
        let user_id = Self::get_user_id_by_username(&action.username)?;

        ListeningEventService::create_event(NewListeningEvent {
            user_id,
            device: action.device.clone(),
            podcast_episode_id: podcast_episode.episode_id.clone(),
            podcast_id: Self::parse_id(&podcast_episode.podcast_id)?,
            podcast_episode_db_id: Self::parse_id(&podcast_episode.id)?,
            delta_seconds: delta,
            start_position: position.saturating_sub(delta),
            end_position: position,
            listened_at: action.timestamp,
        })?;
        Ok(())
    }

    fn find_podcast_episode_for_action(
        action: &podfetch_domain::episode::Episode,
    ) -> Result<Option<PodcastEpisode>, CustomError> {
        use crate::usecases::podcast_episode::PodcastEpisodeUseCase;

        if let Some(guid) = action.guid.as_deref().filter(|g| !g.is_empty())
            && let Some(found) = PodcastEpisodeUseCase::get_podcast_episode_by_guid(guid)?
        {
            return Ok(Some(found));
        }
        PodcastEpisodeUseCase::get_podcast_episode_by_url(&action.episode, None)
    }

    /// Seconds listened for a GPodder `play` action.
    ///
    /// A play action describes one playback session from `started` to
    /// `position`. Clients that omit `started` are measured against the last
    /// known position of the episode instead.
    fn calculate_gpodder_delta(
        started: Option<i32>,
        previous_position: Option<i32>,
        position: i32,
        total: Option<i32>,
    ) -> i32 {
        let Some(from) = started.filter(|s| *s >= 0).or(previous_position) else {
            return 0;
        };
        let delta = position.saturating_sub(from).max(0);
        match total {
            Some(total) if total > 0 => delta.min(total),
            _ => delta,
        }
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
                items
                    .into_iter()
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
        let user_id = Self::get_user_id_by_username(username)?;
        ListeningEventService::default_service().delete_by_user_id(user_id)?;
        Self::repo()
            .delete_by_username(username)
            .map_err(Into::into)
    }

    pub fn delete_watchtime(podcast_id: Uuid) -> Result<(), CustomError> {
        ListeningEventService::default_service().delete_by_podcast_id(podcast_id)?;
        Self::repo()
            .delete_by_podcast_id(podcast_id)
            .map_err(Into::into)
    }

    pub fn delete_by_username(username: &str) -> Result<(), CustomError> {
        let user_id = Self::get_user_id_by_username(username)?;
        ListeningEventService::default_service().delete_by_user_id(user_id)?;
        Self::repo()
            .delete_by_username(username)
            .map_err(Into::into)
    }

    pub fn log_watchtime(
        podcast_episode_id: &str,
        watch_time: i32,
        username: String,
    ) -> Result<(), CustomError> {
        let found_episode =
            crate::usecases::podcast_episode::PodcastEpisodeUseCase::get_podcast_episode_by_id(
                podcast_episode_id,
            )?;

        let Some(found_episode) = found_episode else {
            return Err(CustomErrorInner::NotFound(Warning).into());
        };

        let podcast_uuid = Self::parse_id(&found_episode.podcast_id)?;
        let podcast = crate::services::podcast::service::PodcastService::get_podcast(podcast_uuid)?;

        let now = Utc::now().naive_utc();
        let user_id = Self::get_user_id_by_username(&username)?;

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
                        user_id,
                        device: DEFAULT_DEVICE.to_string(),
                        podcast_episode_id: found_episode.episode_id.clone(),
                        podcast_id: podcast_uuid,
                        podcast_episode_db_id: Self::parse_id(&found_episode.id)?,
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

    /// Mark every episode of a podcast as played (`position == total`) or
    /// unplayed (`position = 0`) for the user. The actions are recorded for
    /// the web device, so GPodder clients pick them up on their next sync.
    /// No listening events are created - nothing was actually listened to.
    pub fn set_podcast_episodes_played(
        podcast_id: uuid::Uuid,
        username: &str,
        played: bool,
    ) -> Result<usize, CustomError> {
        let podcast = crate::services::podcast::service::PodcastService::get_podcast(podcast_id)?;
        let episodes =
            crate::usecases::podcast_episode::PodcastEpisodeUseCase::get_episodes_by_podcast_id(
                podcast_id,
            )?;
        let now = Utc::now().naive_utc();
        let repo = Self::repo();
        let mut changed = 0;

        for episode in episodes {
            let total = episode.total_time.max(0);
            let position = if played { total } else { 0 };
            match Self::get_watchlog_by_device_and_episode(username, &episode.guid, DEFAULT_DEVICE)?
            {
                Some(existing) => {
                    if existing.position == Some(position) && existing.total == Some(total) {
                        continue;
                    }
                    repo.update_progress(existing.id, position, total, now)
                        .map_err(CustomError::from)?;
                }
                None => {
                    if !played {
                        // Never started on this device: nothing to reset
                        // unless another device has progress.
                        if Self::repo()
                            .find_by_username_and_guid(username, &episode.guid)
                            .map_err(CustomError::from)?
                            .is_none()
                        {
                            continue;
                        }
                    }
                    repo.create(NewEpisode {
                        username: username.to_string(),
                        device: DEFAULT_DEVICE.to_string(),
                        podcast: podcast.rssfeed.clone(),
                        episode: episode.url.clone(),
                        timestamp: now,
                        guid: Some(episode.guid.clone()),
                        action: "play".to_string(),
                        started: Some(position),
                        position: Some(position),
                        total: Some(total),
                    })
                    .map_err(CustomError::from)?;
                }
            }
            changed += 1;
        }
        Ok(changed)
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

    fn get_last_watched(
        &self,
        user: &User,
        server_url: &str,
    ) -> Result<Vec<Self::LastWatchedItem>, Self::Error> {
        Self::get_last_watched_episodes(user).map(|items| {
            items
                .into_iter()
                .map(|(podcast_episode, episode, podcast)| {
                    PodcastWatchedEpisodeModelWithPodcastEpisode {
                        podcast_episode: PodcastEpisodeDto::from_episode_with_user(
                            podcast_episode,
                            Some(user.clone()),
                            None::<FavoritePodcastEpisode>,
                            server_url,
                        ),
                        podcast: map_podcast_to_dto(podcast.into(), server_url),
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
    fn gpodder_delta_uses_started_position() {
        assert_eq!(
            WatchtimeUseCase::calculate_gpodder_delta(Some(60), Some(10), 180, Some(300)),
            120
        );
    }

    #[test]
    fn gpodder_delta_falls_back_to_previous_position() {
        assert_eq!(
            WatchtimeUseCase::calculate_gpodder_delta(None, Some(100), 160, None),
            60
        );
    }

    #[test]
    fn gpodder_delta_is_zero_without_reference_or_on_seek_back() {
        assert_eq!(
            WatchtimeUseCase::calculate_gpodder_delta(None, None, 160, None),
            0
        );
        assert_eq!(
            WatchtimeUseCase::calculate_gpodder_delta(Some(200), None, 100, None),
            0
        );
    }

    #[test]
    fn gpodder_delta_is_capped_by_total() {
        assert_eq!(
            WatchtimeUseCase::calculate_gpodder_delta(Some(0), None, 5000, Some(300)),
            300
        );
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
