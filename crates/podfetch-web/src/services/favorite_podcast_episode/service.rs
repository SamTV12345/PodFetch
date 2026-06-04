use common_infrastructure::error::CustomError;
use podfetch_domain::favorite_podcast_episode::{
    FavoritePodcastEpisode, FavoritePodcastEpisodeRepository,
};
use podfetch_persistence::adapters::FavoritePodcastEpisodeRepositoryImpl;
use podfetch_persistence::db::database;
use std::sync::Arc;
use uuid::Uuid;

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

    pub fn get_by_user_id_and_episode_id(
        &self,
        user_id: Uuid,
        episode_id: Uuid,
    ) -> Result<Option<FavoritePodcastEpisode>, CustomError> {
        self.repository
            .get_by_user_id_and_episode_id(user_id, episode_id)
    }

    pub fn set_favorite(
        &self,
        user_id: Uuid,
        episode_id: Uuid,
        favorite: bool,
    ) -> Result<(), CustomError> {
        self.repository
            .save_or_update(FavoritePodcastEpisode::new(user_id, episode_id, favorite))
    }

    pub fn is_liked_by_someone(&self, episode_id: Uuid) -> Result<bool, CustomError> {
        self.repository.is_liked_by_someone(episode_id)
    }

    pub fn get_favorites_by_user_id(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<FavoritePodcastEpisode>, CustomError> {
        self.repository.get_favorites_by_user_id(user_id)
    }
}
