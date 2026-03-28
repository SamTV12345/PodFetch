use chrono::NaiveDateTime;

use crate::episode::Episode;
use crate::favorite_podcast_episode::FavoritePodcastEpisode;

/// A podcast episode - technology-agnostic domain entity.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PodcastEpisode {
    pub id: i32,
    pub podcast_id: i32,
    pub episode_id: String,
    pub name: String,
    pub url: String,
    pub date_of_recording: String,
    pub image_url: String,
    pub total_time: i32,
    pub description: String,
    pub download_time: Option<NaiveDateTime>,
    pub guid: String,
    pub deleted: bool,
    pub file_episode_path: Option<String>,
    pub file_image_path: Option<String>,
    pub episode_numbering_processed: bool,
    pub download_location: Option<String>,
}

impl PodcastEpisode {
    pub fn is_downloaded(&self) -> bool {
        self.download_location.is_some()
    }
}

/// Data for creating a new podcast episode.
#[derive(Debug, Clone)]
pub struct NewPodcastEpisode {
    pub podcast_id: i32,
    pub episode_id: String,
    pub name: String,
    pub url: String,
    pub date_of_recording: String,
    pub image_url: String,
    pub total_time: i32,
    pub description: String,
    pub guid: String,
}

/// Result type for paginated episode queries with history and favorites.
pub type PodcastEpisodeWithHistory = Vec<(PodcastEpisode, Option<Episode>, Option<FavoritePodcastEpisode>)>;

/// Repository trait for PodcastEpisode persistence operations.
pub trait PodcastEpisodeRepository: Send + Sync {
    type Error;

    // Basic CRUD operations
    fn create(&self, episode: NewPodcastEpisode) -> Result<PodcastEpisode, Self::Error>;
    fn find_by_id(&self, id: i32) -> Result<Option<PodcastEpisode>, Self::Error>;
    fn find_by_episode_id(&self, episode_id: &str) -> Result<Option<PodcastEpisode>, Self::Error>;
    fn find_by_url(
        &self,
        url: &str,
        podcast_id: Option<i32>,
    ) -> Result<Option<PodcastEpisode>, Self::Error>;
    fn find_by_guid(&self, guid: &str) -> Result<Option<PodcastEpisode>, Self::Error>;
    fn find_by_podcast_id(&self, podcast_id: i32) -> Result<Vec<PodcastEpisode>, Self::Error>;
    fn find_by_file_path(&self, path: &str) -> Result<Option<PodcastEpisode>, Self::Error>;
    fn update(&self, episode: &PodcastEpisode) -> Result<(), Self::Error>;
    fn delete(&self, id: i32) -> Result<(), Self::Error>;
    fn delete_by_podcast_id(&self, podcast_id: i32) -> Result<(), Self::Error>;

    // Query by URL with LIKE pattern matching
    fn query_by_url_like(&self, url_pattern: &str) -> Result<Option<PodcastEpisode>, Self::Error>;

    // Pagination methods
    fn get_nth_page(
        &self,
        last_id: i32,
        limit: i64,
    ) -> Result<Vec<PodcastEpisode>, Self::Error>;

    /// Get episodes of a podcast with watch history and favorites for a user.
    /// Returns episodes with optional history and favorites, paginated by date_of_recording.
    fn get_episodes_with_history(
        &self,
        podcast_id: i32,
        username: &str,
        last_date: Option<&str>,
        only_unlistened: bool,
        limit: i64,
    ) -> Result<PodcastEpisodeWithHistory, Self::Error>;

    // Get position of episode in podcast (count of episodes with date <= given date)
    fn get_position_of_episode(
        &self,
        timestamp: &str,
        podcast_id: i32,
    ) -> Result<usize, Self::Error>;

    // Get last N episodes by date
    fn get_last_n_episodes(
        &self,
        podcast_id: i32,
        n: i64,
    ) -> Result<Vec<PodcastEpisode>, Self::Error>;

    // Get all episodes
    fn get_all(&self) -> Result<Vec<PodcastEpisode>, Self::Error>;

    // Check if episode is downloaded
    fn check_if_downloaded(&self, url: &str) -> Result<bool, Self::Error>;

    // Get episodes older than N days for a podcast
    fn get_episodes_older_than_days(
        &self,
        days: i64,
        podcast_id: i32,
    ) -> Result<Vec<PodcastEpisode>, Self::Error>;

    // Get top K episodes per podcast (latest K per podcast)
    fn get_episodes_by_podcast_to_k(&self, top_k: i64) -> Result<Vec<PodcastEpisode>, Self::Error>;

    // Update methods
    fn update_local_paths(
        &self,
        episode_id: &str,
        file_image_path: &str,
        file_episode_path: &str,
    ) -> Result<(), Self::Error>;

    fn update_download_status(
        &self,
        url: &str,
        download_location: Option<String>,
        download_time: NaiveDateTime,
    ) -> Result<PodcastEpisode, Self::Error>;

    fn remove_download_status(&self, id: i32) -> Result<(), Self::Error>;

    fn update_guid(&self, episode_id: &str, guid: &str) -> Result<(), Self::Error>;

    fn update_deleted(&self, episode_id: &str, deleted: bool) -> Result<usize, Self::Error>;

    fn update_episode_numbering_processed(
        &self,
        episode_id: &str,
        processed: bool,
    ) -> Result<(), Self::Error>;
}
