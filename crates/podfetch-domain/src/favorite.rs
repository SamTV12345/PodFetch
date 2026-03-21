/// A user's favorite status for a podcast - technology-agnostic domain entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Favorite {
    pub username: String,
    pub podcast_id: i32,
    pub favored: bool,
}

impl Favorite {
    pub fn new(username: String, podcast_id: i32, favored: bool) -> Self {
        Self {
            username,
            podcast_id,
            favored,
        }
    }
}

/// Repository trait for Favorite persistence operations.
pub trait FavoriteRepository: Send + Sync {
    type Error;

    fn upsert(&self, favorite: Favorite) -> Result<(), Self::Error>;
    fn find_by_username_and_podcast_id(
        &self,
        username: &str,
        podcast_id: i32,
    ) -> Result<Option<Favorite>, Self::Error>;
    fn find_favored_by_username(&self, username: &str) -> Result<Vec<Favorite>, Self::Error>;
    fn delete_by_username(&self, username: &str) -> Result<(), Self::Error>;
}
