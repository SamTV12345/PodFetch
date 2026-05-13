use common_infrastructure::error::CustomError;
use podfetch_domain::audiobookshelf::listening_session::{
    ListeningSession, ListeningSessionRepository,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct AudiobookshelfListeningSessionService {
    repository: Arc<dyn ListeningSessionRepository<Error = CustomError>>,
}

impl AudiobookshelfListeningSessionService {
    pub fn new(repository: Arc<dyn ListeningSessionRepository<Error = CustomError>>) -> Self {
        Self { repository }
    }

    pub fn create(&self, session: ListeningSession) -> Result<ListeningSession, CustomError> {
        self.repository.create(session)
    }

    pub fn list_for_user(
        &self,
        user_id: i32,
        limit: i64,
    ) -> Result<Vec<ListeningSession>, CustomError> {
        self.repository.list_for_user(user_id, limit)
    }
}
