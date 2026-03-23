use crate::favorite::Favorite;

/// A podcast in the system - technology-agnostic domain entity.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Podcast {
    pub id: i32,
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
}

/// Data for updating podcast metadata (from RSS feed parsing).
#[derive(Debug, Clone)]
pub struct PodcastMetadataUpdate {
    pub id: i32,
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
    fn find_by_id(&self, id: i32) -> Result<Option<Podcast>, Self::Error>;
    fn find_by_rss_feed(&self, rss_feed: &str) -> Result<Option<Podcast>, Self::Error>;
    fn find_by_directory_id(&self, directory_id: &str) -> Result<Option<Podcast>, Self::Error>;
    fn find_by_track_id(&self, track_id: i32) -> Result<Option<Podcast>, Self::Error>;
    fn find_by_image_path(&self, path: &str) -> Result<Option<Podcast>, Self::Error>;
    fn find_by_episode_id(&self, episode_id: i32) -> Result<Option<Podcast>, Self::Error>;
    fn find_all(&self) -> Result<Vec<Podcast>, Self::Error>;
    fn find_all_with_favorites(
        &self,
        username: &str,
    ) -> Result<Vec<PodcastWithFavorite>, Self::Error>;
    fn update_metadata(&self, update: PodcastMetadataUpdate) -> Result<(), Self::Error>;
    fn update_active(&self, id: i32, active: bool) -> Result<(), Self::Error>;
    fn update_name(&self, id: i32, name: &str) -> Result<(), Self::Error>;
    fn update_rss_feed(&self, id: i32, rss_feed: &str) -> Result<(), Self::Error>;
    fn update_original_image_url(&self, id: i32, image_url: &str) -> Result<(), Self::Error>;
    fn update_image_url_and_download_location(
        &self,
        directory_id: &str,
        image_url: &str,
        download_location: &str,
    ) -> Result<(), Self::Error>;
    fn delete(&self, id: i32) -> Result<(), Self::Error>;
}
