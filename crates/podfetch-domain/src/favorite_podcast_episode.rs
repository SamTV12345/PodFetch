#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FavoritePodcastEpisode {
    pub user_id: i32,
    pub episode_id: i32,
    pub favorite: bool,
}

impl FavoritePodcastEpisode {
    pub fn new(user_id: i32, episode_id: i32, favorite: bool) -> Self {
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
        user_id: i32,
        episode_id: i32,
    ) -> Result<Option<FavoritePodcastEpisode>, Self::Error>;

    fn save_or_update(&self, favorite: FavoritePodcastEpisode) -> Result<(), Self::Error>;

    fn is_liked_by_someone(&self, episode_id: i32) -> Result<bool, Self::Error>;

    fn get_favorites_by_user_id(
        &self,
        user_id: i32,
    ) -> Result<Vec<FavoritePodcastEpisode>, Self::Error>;
}
