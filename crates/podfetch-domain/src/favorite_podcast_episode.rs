#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FavoritePodcastEpisode {
    pub username: String,
    pub episode_id: i32,
    pub favorite: bool,
}

impl FavoritePodcastEpisode {
    pub fn new(username: impl Into<String>, episode_id: i32, favorite: bool) -> Self {
        Self {
            username: username.into(),
            episode_id,
            favorite,
        }
    }
}

pub trait FavoritePodcastEpisodeRepository: Send + Sync {
    type Error;

    fn get_by_username_and_episode_id(
        &self,
        username: &str,
        episode_id: i32,
    ) -> Result<Option<FavoritePodcastEpisode>, Self::Error>;

    fn save_or_update(&self, favorite: FavoritePodcastEpisode) -> Result<(), Self::Error>;

    fn is_liked_by_someone(&self, episode_id: i32) -> Result<bool, Self::Error>;
}
