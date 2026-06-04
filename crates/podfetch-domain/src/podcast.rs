use crate::favorite::Favorite;
use uuid::Uuid;

/// A podcast in the system - technology-agnostic domain entity.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Podcast {
    pub id: Uuid,
    pub legacy_id: Option<i64>,
    pub name: String,
    pub directory_id: String,
    pub rssfeed: String,
    pub image_url: String,
    pub summary: Option<String>,
    pub language: Option<String>,
    pub explicit: Option<String>,
    pub keywords: Option<String>,
    pub last_build_date: Option<String>,
    pub author: Option<String>,
    pub active: bool,
    pub original_image_url: String,
    pub directory_name: String,
    pub download_location: Option<String>,
    pub guid: Option<String>,
    pub added_by: Option<Uuid>,
}

/// A podcast with its optional favorite status for a specific user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PodcastWithFavorite {
    pub podcast: Podcast,
    pub favorite: Option<Favorite>,
}

/// Data for creating a new podcast.
#[derive(Debug, Clone)]
pub struct NewPodcast {
    pub name: String,
    pub directory_id: String,
    pub rssfeed: String,
    pub image_url: String,
    pub directory_name: String,
    pub added_by: Option<Uuid>,
}

/// Data for updating podcast metadata (from RSS feed parsing).
#[derive(Debug, Clone)]
pub struct PodcastMetadataUpdate {
    pub id: Uuid,
    pub author: Option<String>,
    pub keywords: Option<String>,
    pub explicit: Option<String>,
    pub language: Option<String>,
    pub description: Option<String>,
    pub last_build_date: Option<String>,
    pub guid: Option<String>,
}

/// Repository trait for Podcast persistence operations.
pub trait PodcastRepository: Send + Sync {
    type Error;

    fn create(&self, podcast: NewPodcast) -> Result<Podcast, Self::Error>;
    fn find_by_id(&self, id: Uuid) -> Result<Option<Podcast>, Self::Error>;
    /// Resolve a podcast by its pre-migration integer id (backwards-compat for
    /// durable RSS/episode links). Returns `None` for rows created after the
    /// UUID migration (which have no `legacy_id`).
    fn find_by_legacy_id(&self, legacy_id: i64) -> Result<Option<Podcast>, Self::Error>;
    fn find_by_rss_feed(&self, rss_feed: &str) -> Result<Option<Podcast>, Self::Error>;
    fn find_by_directory_id(&self, directory_id: &str) -> Result<Option<Podcast>, Self::Error>;
    fn find_by_track_id(&self, track_id: i32) -> Result<Option<Podcast>, Self::Error>;
    fn find_by_image_path(&self, path: &str) -> Result<Option<Podcast>, Self::Error>;
    fn find_by_episode_id(&self, episode_id: Uuid) -> Result<Option<Podcast>, Self::Error>;
    fn find_all(&self) -> Result<Vec<Podcast>, Self::Error>;
    fn find_all_with_favorites(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<PodcastWithFavorite>, Self::Error>;
    fn update_metadata(&self, update: PodcastMetadataUpdate) -> Result<(), Self::Error>;
    fn update_active(&self, id: Uuid, active: bool) -> Result<(), Self::Error>;
    fn update_name(&self, id: Uuid, name: &str) -> Result<(), Self::Error>;
    fn update_rss_feed(&self, id: Uuid, rss_feed: &str) -> Result<(), Self::Error>;
    fn update_original_image_url(&self, id: Uuid, image_url: &str) -> Result<(), Self::Error>;
    fn update_image_url_and_download_location(
        &self,
        directory_id: &str,
        image_url: &str,
        download_location: &str,
    ) -> Result<(), Self::Error>;
    fn delete(&self, id: Uuid) -> Result<(), Self::Error>;
    fn count_by_added_by(&self, user_id: Uuid) -> Result<i64, Self::Error>;
}
