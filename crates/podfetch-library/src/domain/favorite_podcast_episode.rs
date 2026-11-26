use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FavoritePodcastEpisode {
    pub user_id: Uuid,
    pub episode_id: Uuid,
    pub favorite: bool,
}

impl FavoritePodcastEpisode {
    pub fn new(user_id: Uuid, episode_id: Uuid, favorite: bool) -> Self {
        Self {
            user_id,
            episode_id,
            favorite,
        }
    }
}

pub trait FavoritePodcastEpisodeRepository: Send + Sync {
    type Error;

    fn get_by_user_id_and_episode_id(
        &self,
        user_id: Uuid,
        episode_id: Uuid,
    ) -> Result<Option<FavoritePodcastEpisode>, Self::Error>;

    fn save_or_update(&self, favorite: FavoritePodcastEpisode) -> Result<(), Self::Error>;

    fn is_liked_by_someone(&self, episode_id: Uuid) -> Result<bool, Self::Error>;

    fn get_favorites_by_user_id(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<FavoritePodcastEpisode>, Self::Error>;

    /// The user's favorite marks on the given episodes.
    fn get_by_user_id_and_episode_ids(
        &self,
        user_id: Uuid,
        episode_ids: &[Uuid],
    ) -> Result<Vec<FavoritePodcastEpisode>, Self::Error>;

    /// Remove every user's favorite marks on the given episodes.
    /// `favorite_podcast_episodes.episode_id` has no `ON DELETE` action, so
    /// this has to run before the episodes are deleted.
    fn delete_by_episode_ids(&self, episode_ids: &[Uuid]) -> Result<usize, Self::Error>;
}
