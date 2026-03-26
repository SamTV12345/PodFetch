use podfetch_persistence::db::database;
use podfetch_persistence::adapters::FavoritePodcastEpisodeRepositoryImpl;
use common_infrastructure::error::CustomError;
use podfetch_domain::favorite_podcast_episode::{
    FavoritePodcastEpisode, FavoritePodcastEpisodeRepository,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct FavoritePodcastEpisodeService {
    repository: Arc<dyn FavoritePodcastEpisodeRepository<Error = CustomError>>,
}

impl FavoritePodcastEpisodeService {
    pub fn new(repository: Arc<dyn FavoritePodcastEpisodeRepository<Error = CustomError>>) -> Self {
        Self { repository }
    }

    pub fn default_service() -> Self {
        Self::new(Arc::new(FavoritePodcastEpisodeRepositoryImpl::new(
            database(),
        )))
    }

    pub fn get_by_username_and_episode_id(
        &self,
        username: &str,
        episode_id: i32,
    ) -> Result<Option<FavoritePodcastEpisode>, CustomError> {
        self.repository
            .get_by_username_and_episode_id(username, episode_id)
    }

    pub fn set_favorite(
        &self,
        username: &str,
        episode_id: i32,
        favorite: bool,
    ) -> Result<(), CustomError> {
        self.repository
            .save_or_update(FavoritePodcastEpisode::new(username, episode_id, favorite))
    }

    pub fn is_liked_by_someone(&self, episode_id: i32) -> Result<bool, CustomError> {
        self.repository.is_liked_by_someone(episode_id)
    }
}

