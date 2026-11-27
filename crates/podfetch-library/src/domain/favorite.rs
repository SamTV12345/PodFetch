use uuid::Uuid;

/// A user's favorite status for a podcast - technology-agnostic domain entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Favorite {
    pub user_id: Uuid,
    pub podcast_id: Uuid,
    pub favored: bool,
}

impl Favorite {
    pub fn new(user_id: Uuid, podcast_id: Uuid, favored: bool) -> Self {
        Self {
            user_id,
            podcast_id,
            favored,
        }
    }
}

/// Repository trait for Favorite persistence operations.
pub trait FavoriteRepository: Send + Sync {
    type Error;

    fn upsert(&self, favorite: Favorite) -> Result<(), Self::Error>;
    fn find_by_user_id_and_podcast_id(
        &self,
        user_id: Uuid,
        podcast_id: Uuid,
    ) -> Result<Option<Favorite>, Self::Error>;
    fn find_favored_by_user_id(&self, user_id: Uuid) -> Result<Vec<Favorite>, Self::Error>;
    fn delete_by_user_id(&self, user_id: Uuid) -> Result<(), Self::Error>;
    fn delete_by_podcast_id(&self, podcast_id: Uuid) -> Result<(), Self::Error>;

    /// Update or insert a favorite status for a podcast.
    fn update_podcast_favor(
        &self,
        podcast_id: Uuid,
        favor: bool,
        user_id: Uuid,
    ) -> Result<(), Self::Error>;

    /// The user's favorite rows for the given podcasts.
    fn find_by_user_and_podcasts(
        &self,
        user_id: Uuid,
        podcast_ids: &[Uuid],
    ) -> Result<Vec<Favorite>, Self::Error>;
}
