use common_infrastructure::error::CustomError;
use podfetch_domain::favorite_podcast_episode::{
    FavoritePodcastEpisode, FavoritePodcastEpisodeRepository,
};
use podfetch_persistence::db::Database;
use podfetch_persistence::favorite_podcast_episode::DieselFavoritePodcastEpisodeRepository;

pub struct FavoritePodcastEpisodeRepositoryImpl {
    inner: DieselFavoritePodcastEpisodeRepository,
}

impl FavoritePodcastEpisodeRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselFavoritePodcastEpisodeRepository::new(database),
        }
    }
}

impl FavoritePodcastEpisodeRepository for FavoritePodcastEpisodeRepositoryImpl {
    type Error = CustomError;

    fn get_by_username_and_episode_id(
        &self,
        username: &str,
        episode_id: i32,
    ) -> Result<Option<FavoritePodcastEpisode>, Self::Error> {
        self.inner
            .get_by_username_and_episode_id(username, episode_id)
            .map_err(Into::into)
    }

    fn save_or_update(&self, favorite: FavoritePodcastEpisode) -> Result<(), Self::Error> {
        self.inner.save_or_update(favorite).map_err(Into::into)
    }

    fn is_liked_by_someone(&self, episode_id: i32) -> Result<bool, Self::Error> {
        self.inner
            .is_liked_by_someone(episode_id)
            .map_err(Into::into)
    }
}

