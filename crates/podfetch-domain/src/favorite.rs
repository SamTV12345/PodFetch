use crate::ordering::{OrderCriteria, OrderOption};
use crate::podcast::Podcast;
use crate::tag::Tag;

/// A user's favorite status for a podcast - technology-agnostic domain entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Favorite {
    pub user_id: i32,
    pub podcast_id: i32,
    pub favored: bool,
}

impl Favorite {
    pub fn new(user_id: i32, podcast_id: i32, favored: bool) -> Self {
        Self {
            user_id,
            podcast_id,
            favored,
        }
    }
}

/// Result type for podcast search with favorite status and tags.
#[derive(Debug, Clone)]
pub struct PodcastSearchResult {
    pub podcast: Podcast,
    pub favorite: Option<Favorite>,
    pub tags: Vec<Tag>,
}

/// Result type for favored podcast search with tags.
#[derive(Debug, Clone)]
pub struct FavoredPodcastSearchResult {
    pub podcast: Podcast,
    pub favorite: Favorite,
    pub tags: Vec<Tag>,
}

/// Result type for podcast with favorite status.
#[derive(Debug, Clone)]
pub struct PodcastWithFavorite {
    pub podcast: Podcast,
    pub favorite: Favorite,
}

/// Repository trait for Favorite persistence operations.
pub trait FavoriteRepository: Send + Sync {
    type Error;

    fn upsert(&self, favorite: Favorite) -> Result<(), Self::Error>;
    fn find_by_user_id_and_podcast_id(
        &self,
        user_id: i32,
        podcast_id: i32,
    ) -> Result<Option<Favorite>, Self::Error>;
    fn find_favored_by_user_id(&self, user_id: i32) -> Result<Vec<Favorite>, Self::Error>;
    fn delete_by_user_id(&self, user_id: i32) -> Result<(), Self::Error>;

    /// Update or insert a favorite status for a podcast.
    fn update_podcast_favor(
        &self,
        podcast_id: i32,
        favor: bool,
        user_id: i32,
    ) -> Result<(), Self::Error>;

    /// Get all favored podcasts for a user with their favorite status.
    fn get_favored_podcasts(&self, user_id: i32)
    -> Result<Vec<PodcastWithFavorite>, Self::Error>;

    /// Search for favored podcasts with tags.
    fn search_podcasts_favored(
        &self,
        order: OrderCriteria,
        title: Option<String>,
        order_option: OrderOption,
        user_id: i32,
    ) -> Result<Vec<FavoredPodcastSearchResult>, Self::Error>;

    /// Search for all podcasts with optional favorite status and tags.
    fn search_podcasts(
        &self,
        order: OrderCriteria,
        title: Option<String>,
        order_option: OrderOption,
        user_id: i32,
    ) -> Result<Vec<PodcastSearchResult>, Self::Error>;
}
