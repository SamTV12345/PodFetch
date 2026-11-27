//! Fetching SponsorBlock data for a downloaded episode, and the segments
//! and preferences the player asks for.

use super::client::{FetchedSegment, SponsorblockClient};
use super::domain::{SkipPreferences, SponsorSegment, SponsorblockRepository};
use common_infrastructure::error::CustomError;
use podfetch_kernel::UserId;
use std::sync::Arc;

/// Port to the settings: whether the admin turned SponsorBlock on.
pub trait SponsorblockSwitch: Send + Sync {
    fn enabled(&self) -> Result<bool, CustomError>;
}

/// What the media library needs to know about a catalog episode to look it
/// up on SponsorBlock.
#[derive(Debug, Clone, Copy)]
pub struct EpisodeToCheck<'a> {
    pub episode_id: &'a str,
    /// Only YouTube episodes have SponsorBlock data.
    pub youtube_video_id: Option<&'a str>,
    /// Length of the episode in seconds; 0 if unknown.
    pub duration_secs: i64,
}

/// Tolerance for declaring a duration mismatch: the larger of 2 seconds or 1%.
fn durations_mismatch(episode_secs: i64, sb_secs: f64) -> bool {
    // If either side is unknown (<= 0) we cannot verify alignment, so we do NOT
    // flag — better to skip than to disable the feature for feeds without a
    // duration. Only a confident, sizeable divergence flags as a mismatch.
    if episode_secs <= 0 || sb_secs <= 0.0 {
        return false;
    }
    let diff = (episode_secs as f64 - sb_secs).abs();
    let tolerance = (sb_secs * 0.01).max(2.0);
    diff > tolerance
}

pub struct SponsorblockService {
    repository: Arc<dyn SponsorblockRepository>,
    client: SponsorblockClient,
    switch: Arc<dyn SponsorblockSwitch>,
}

impl SponsorblockService {
    pub fn new(
        repository: Arc<dyn SponsorblockRepository>,
        client: SponsorblockClient,
        switch: Arc<dyn SponsorblockSwitch>,
    ) -> Self {
        Self {
            repository,
            client,
            switch,
        }
    }

    /// Fetches and stores the SponsorBlock segments of one episode,
    /// replacing the stored ones. Returns the number of segments stored;
    /// 0 if SponsorBlock is off or the episode isn't from YouTube.
    pub async fn fetch_and_store(&self, episode: EpisodeToCheck<'_>) -> Result<usize, CustomError> {
        if !self.switch.enabled()? {
            return Ok(0);
        }
        let Some(video_id) = episode.youtube_video_id else {
            return Ok(0);
        };

        let fetched: Vec<FetchedSegment> = self.client.fetch_segments(video_id).await?;

        let now = chrono::Utc::now().naive_utc();
        let segments: Vec<SponsorSegment> = fetched
            .into_iter()
            .map(|seg| SponsorSegment {
                duration_mismatch: durations_mismatch(
                    episode.duration_secs,
                    seg.video_duration_secs,
                ),
                uuid: seg.uuid,
                category: seg.category,
                action_type: seg.action_type,
                start_ms: seg.start_ms,
                end_ms: seg.end_ms,
                votes: seg.votes,
                locked: seg.locked,
                fetched_at: now,
            })
            .collect();

        self.repository
            .replace_segments(episode.episode_id, &segments)?;
        Ok(segments.len())
    }

    /// Forgets the segments of episodes that leave the catalog.
    pub fn forget_episodes(&self, episode_ids: &[String]) -> Result<(), CustomError> {
        self.repository.forget_segments(episode_ids)
    }

    /// The episode's segments, ordered by start.
    pub fn segments(&self, episode_id: &str) -> Result<Vec<SponsorSegment>, CustomError> {
        self.repository.segments(episode_id)
    }

    /// The user's preferences, or the defaults if they never saved any.
    pub fn preferences(&self, user_id: UserId) -> Result<SkipPreferences, CustomError> {
        Ok(self.repository.preferences(user_id)?.unwrap_or_default())
    }

    pub fn save_preferences(
        &self,
        user_id: UserId,
        preferences: &SkipPreferences,
    ) -> Result<(), CustomError> {
        self.repository.save_preferences(user_id, preferences)
    }

    pub fn delete_preferences(&self, user_id: UserId) -> Result<(), CustomError> {
        self.repository.delete_preferences(user_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MemoryRepository {
        segments: Mutex<Vec<(String, SponsorSegment)>>,
        preferences: Mutex<Vec<(UserId, SkipPreferences)>>,
    }

    impl SponsorblockRepository for MemoryRepository {
        fn replace_segments(
            &self,
            episode_id: &str,
            segments: &[SponsorSegment],
        ) -> Result<(), CustomError> {
            let mut stored = self.segments.lock().unwrap();
            stored.retain(|(id, _)| id != episode_id);
            stored.extend(segments.iter().map(|s| (episode_id.to_string(), s.clone())));
            Ok(())
        }

        fn forget_segments(&self, episode_ids: &[String]) -> Result<(), CustomError> {
            self.segments
                .lock()
                .unwrap()
                .retain(|(id, _)| !episode_ids.contains(id));
            Ok(())
        }

        fn segments(&self, episode_id: &str) -> Result<Vec<SponsorSegment>, CustomError> {
            Ok(self
                .segments
                .lock()
                .unwrap()
                .iter()
                .filter(|(id, _)| id == episode_id)
                .map(|(_, s)| s.clone())
                .collect())
        }

        fn preferences(&self, user_id: UserId) -> Result<Option<SkipPreferences>, CustomError> {
            Ok(self
                .preferences
                .lock()
                .unwrap()
                .iter()
                .find(|(id, _)| *id == user_id)
                .map(|(_, p)| p.clone()))
        }

        fn save_preferences(
            &self,
            user_id: UserId,
            preferences: &SkipPreferences,
        ) -> Result<(), CustomError> {
            let mut stored = self.preferences.lock().unwrap();
            stored.retain(|(id, _)| *id != user_id);
            stored.push((user_id, preferences.clone()));
            Ok(())
        }

        fn delete_preferences(&self, user_id: UserId) -> Result<(), CustomError> {
            self.preferences
                .lock()
                .unwrap()
                .retain(|(id, _)| *id != user_id);
            Ok(())
        }
    }

    struct Switch(bool);

    impl SponsorblockSwitch for Switch {
        fn enabled(&self) -> Result<bool, CustomError> {
            Ok(self.0)
        }
    }

    /// One sponsor segment of a 600 s video, for any hash prefix.
    const RESPONSE: &str = r#"[{"videoID":"dQw4w9WgXcQ","segments":[
        {"UUID":"seg-1","category":"sponsor","actionType":"skip","segment":[30.0,45.0],"votes":3,"locked":0,"videoDuration":600.0}
    ]}]"#;

    /// Serves `RESPONSE` and counts the requests.
    async fn mock_sponsorblock() -> (String, Arc<std::sync::atomic::AtomicUsize>) {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let hits = Arc::new(AtomicUsize::new(0));
        let counter = hits.clone();
        let app = axum::Router::new().route(
            "/api/skipSegments/{prefix}",
            axum::routing::get(move || {
                counter.fetch_add(1, Ordering::SeqCst);
                async { RESPONSE }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        (format!("http://{addr}"), hits)
    }

    fn service(base_url: &str, enabled: bool) -> (SponsorblockService, Arc<MemoryRepository>) {
        let repository = Arc::new(MemoryRepository::default());
        let service = SponsorblockService::new(
            repository.clone(),
            SponsorblockClient::with_base_url(reqwest::Client::new(), base_url),
            Arc::new(Switch(enabled)),
        );
        (service, repository)
    }

    fn youtube_episode(duration_secs: i64) -> EpisodeToCheck<'static> {
        EpisodeToCheck {
            episode_id: "e1",
            youtube_video_id: Some("dQw4w9WgXcQ"),
            duration_secs,
        }
    }

    #[tokio::test]
    async fn stores_the_fetched_segments() {
        let (url, _) = mock_sponsorblock().await;
        let (service, _) = service(&url, true);

        assert_eq!(
            service.fetch_and_store(youtube_episode(600)).await.unwrap(),
            1
        );

        let segments = service.segments("e1").unwrap();
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].uuid, "seg-1");
        assert_eq!(segments[0].start_ms, 30_000);
        assert_eq!(segments[0].end_ms, 45_000);
        assert!(!segments[0].duration_mismatch);
    }

    #[tokio::test]
    async fn flags_segments_of_an_episode_with_another_length() {
        let (url, _) = mock_sponsorblock().await;
        let (service, _) = service(&url, true);

        service.fetch_and_store(youtube_episode(540)).await.unwrap();

        assert!(service.segments("e1").unwrap()[0].duration_mismatch);
    }

    #[tokio::test]
    async fn replaces_segments_stored_earlier() {
        let (url, _) = mock_sponsorblock().await;
        let (service, repository) = service(&url, true);
        let stale = SponsorSegment {
            uuid: "stale".into(),
            category: "intro".into(),
            action_type: "skip".into(),
            start_ms: 0,
            end_ms: 1000,
            votes: 0,
            locked: false,
            duration_mismatch: false,
            fetched_at: chrono::Utc::now().naive_utc(),
        };
        repository.replace_segments("e1", &[stale]).unwrap();

        service.fetch_and_store(youtube_episode(600)).await.unwrap();

        let uuids: Vec<String> = service
            .segments("e1")
            .unwrap()
            .into_iter()
            .map(|s| s.uuid)
            .collect();
        assert_eq!(uuids, ["seg-1"]);
    }

    #[tokio::test]
    async fn asks_nobody_when_switched_off() {
        let (url, hits) = mock_sponsorblock().await;
        let (service, _) = service(&url, false);

        assert_eq!(
            service.fetch_and_store(youtube_episode(600)).await.unwrap(),
            0
        );

        assert_eq!(hits.load(std::sync::atomic::Ordering::SeqCst), 0);
        assert!(service.segments("e1").unwrap().is_empty());
    }

    #[tokio::test]
    async fn skips_episodes_not_from_youtube() {
        let (url, hits) = mock_sponsorblock().await;
        let (service, _) = service(&url, true);
        let episode = EpisodeToCheck {
            youtube_video_id: None,
            ..youtube_episode(600)
        };

        assert_eq!(service.fetch_and_store(episode).await.unwrap(), 0);
        assert_eq!(hits.load(std::sync::atomic::Ordering::SeqCst), 0);
    }

    #[test]
    fn preferences_default_until_saved_and_go_with_the_user() {
        let (service, _) = service("http://unused", true);
        let user = UserId::from(uuid::Uuid::new_v4());
        assert_eq!(
            service.preferences(user).unwrap(),
            SkipPreferences::default()
        );

        let saved = SkipPreferences {
            skip_intro: true,
            ..SkipPreferences::default()
        };
        service.save_preferences(user, &saved).unwrap();
        assert_eq!(service.preferences(user).unwrap(), saved);

        service.delete_preferences(user).unwrap();
        assert_eq!(
            service.preferences(user).unwrap(),
            SkipPreferences::default()
        );
    }

    #[test]
    fn unknown_durations_never_mismatch() {
        assert!(!durations_mismatch(0, 600.0));
        assert!(!durations_mismatch(600, 0.0));
    }

    #[test]
    fn close_durations_do_not_mismatch() {
        // 600s vs 601s -> diff 1s, tolerance max(6, 2)=6 -> ok.
        assert!(!durations_mismatch(600, 601.0));
    }

    #[test]
    fn large_divergence_flags_mismatch() {
        // 600s vs 540s -> diff 60s, tolerance 6 -> mismatch.
        assert!(durations_mismatch(600, 540.0));
    }

    #[test]
    fn small_videos_use_two_second_floor() {
        // 100s vs 103s -> diff 3s, tolerance max(1.03, 2)=2 -> mismatch.
        assert!(durations_mismatch(100, 103.0));
        // 100s vs 101s -> diff 1s -> ok.
        assert!(!durations_mismatch(100, 101.0));
    }
}
