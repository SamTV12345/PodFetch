use crate::podcast::Podcast;
use crate::podcast_episode::PodcastEpisode;
use chrono::NaiveDateTime;

/// An episode watch/action log entry - technology-agnostic domain entity.
/// This tracks user actions on episodes (play, download, etc.) for gPodder sync.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Episode {
    pub id: i32,
    pub username: String,
    pub device: String,
    pub podcast: String,
    pub episode: String,
    pub timestamp: NaiveDateTime,
    pub guid: Option<String>,
    pub action: String,
    pub started: Option<i32>,
    pub position: Option<i32>,
    pub total: Option<i32>,
}

/// Data for creating a new episode action log.
#[derive(Debug, Clone)]
pub struct NewEpisode {
    pub username: String,
    pub device: String,
    pub podcast: String,
    pub episode: String,
    pub timestamp: NaiveDateTime,
    pub guid: Option<String>,
    pub action: String,
    pub started: Option<i32>,
    pub position: Option<i32>,
    pub total: Option<i32>,
}

/// Result of a last watched episode query, combining episode action with podcast episode and podcast info.
#[derive(Debug, Clone)]
pub struct LastWatchedEpisode {
    pub podcast_episode: PodcastEpisode,
    pub episode_action: Episode,
    pub podcast: Podcast,
}

/// Repository trait for Episode (watch log) persistence operations.
pub trait EpisodeRepository: Send + Sync {
    type Error;

    /// Create a new episode action log entry.
    fn create(&self, episode: NewEpisode) -> Result<Episode, Self::Error>;

    /// Insert episode action, returning existing if duplicate found (idempotent insert).
    /// Checks for existing entry with same timestamp, device, podcast, and episode URL.
    fn insert_episode(&self, episode: &Episode) -> Result<Episode, Self::Error>;

    /// Find episode action by username and episode URL.
    fn find_by_username_and_episode(
        &self,
        username: &str,
        episode_url: &str,
    ) -> Result<Option<Episode>, Self::Error>;

    /// Find episode action by username, device, and guid.
    fn find_by_username_device_guid(
        &self,
        username: &str,
        device: &str,
        guid: &str,
    ) -> Result<Option<Episode>, Self::Error>;

    /// Find the most recent episode action by username and guid (any device).
    /// Used as fallback matching when episode URLs differ (e.g. local vs original feed).
    fn find_by_username_and_guid(
        &self,
        username: &str,
        guid: &str,
    ) -> Result<Option<Episode>, Self::Error>;

    /// Find all episode actions for a user with optional filters.
    /// If `default_device` is provided, results will include both the specified device
    /// and the default device.
    fn find_actions_by_username(
        &self,
        username: &str,
        since: Option<NaiveDateTime>,
        device: Option<&str>,
        podcast: Option<&str>,
        default_device: Option<&str>,
    ) -> Result<Vec<Episode>, Self::Error>;

    /// Get watch log entry by username and episode URL, joined with podcasts table.
    fn find_watch_log_by_username_and_episode(
        &self,
        username: &str,
        episode_url: &str,
    ) -> Result<Option<Episode>, Self::Error>;

    /// Get the last watched episodes for a user.
    /// Returns podcast episodes with their watch action and podcast info, ordered by most recent first.
    fn find_last_watched_episodes(
        &self,
        username: &str,
    ) -> Result<Vec<LastWatchedEpisode>, Self::Error>;

    /// Get watchtime for a specific episode by episode_id and username.
    fn find_watchtime(
        &self,
        episode_id: &str,
        username: &str,
    ) -> Result<Option<Episode>, Self::Error>;

    /// Update the position/timestamp for an episode action.
    fn update_position(
        &self,
        id: i32,
        position: i32,
        timestamp: NaiveDateTime,
    ) -> Result<(), Self::Error>;

    /// Delete all episode actions for a user.
    fn delete_by_username(&self, username: &str) -> Result<(), Self::Error>;

    /// Delete all episode actions for a podcast feed URL.
    fn delete_by_podcast_feed(&self, podcast_feed: &str) -> Result<(), Self::Error>;

    /// Delete all episode actions for a podcast by ID (looks up the podcast's RSS feed first).
    fn delete_by_podcast_id(&self, podcast_id: i32) -> Result<(), Self::Error>;
}
