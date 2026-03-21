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

/// Repository trait for Episode (watch log) persistence operations.
pub trait EpisodeRepository: Send + Sync {
    type Error;

    fn create(&self, episode: NewEpisode) -> Result<Episode, Self::Error>;
    fn find_by_username_and_episode(
        &self,
        username: &str,
        episode_url: &str,
    ) -> Result<Option<Episode>, Self::Error>;
    fn find_by_username_device_guid(
        &self,
        username: &str,
        device: &str,
        guid: &str,
    ) -> Result<Option<Episode>, Self::Error>;
    fn find_actions_by_username(
        &self,
        username: &str,
        since: Option<NaiveDateTime>,
        device: Option<&str>,
        podcast: Option<&str>,
    ) -> Result<Vec<Episode>, Self::Error>;
    fn update_position(&self, id: i32, position: i32, timestamp: NaiveDateTime)
        -> Result<(), Self::Error>;
    fn delete_by_username(&self, username: &str) -> Result<(), Self::Error>;
    fn delete_by_podcast_feed(&self, podcast_feed: &str) -> Result<(), Self::Error>;
}
