use common_infrastructure::error::CustomError;
use podfetch_domain::audiobookshelf::media_progress::{MediaProgress, MediaProgressRepository};
use std::sync::Arc;

#[derive(Clone)]
pub struct AudiobookshelfMediaProgressService {
    repository: Arc<dyn MediaProgressRepository<Error = CustomError>>,
}

impl AudiobookshelfMediaProgressService {
    pub fn new(repository: Arc<dyn MediaProgressRepository<Error = CustomError>>) -> Self {
        Self { repository }
    }

    pub fn list_for_user(&self, user_id: uuid::Uuid) -> Result<Vec<MediaProgress>, CustomError> {
        self.repository.list_for_user(user_id)
    }

    pub fn find(
        &self,
        user_id: uuid::Uuid,
        library_item_id: &str,
        episode_id: Option<&str>,
    ) -> Result<Option<MediaProgress>, CustomError> {
        self.repository.find(user_id, library_item_id, episode_id)
    }

    pub fn upsert(&self, progress: MediaProgress) -> Result<MediaProgress, CustomError> {
        self.repository.upsert(progress)
    }
}
