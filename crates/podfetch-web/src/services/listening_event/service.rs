use podfetch_persistence::db::database;
use podfetch_persistence::adapters::ListeningEventRepositoryImpl;
use common_infrastructure::error::CustomError;
use chrono::NaiveDateTime;
use podfetch_domain::listening_event::{
    ListeningEvent, ListeningEventRepository, NewListeningEvent,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct ListeningEventService {
    repository: Arc<dyn ListeningEventRepository<Error = CustomError>>,
}

impl ListeningEventService {
    pub fn new(repository: Arc<dyn ListeningEventRepository<Error = CustomError>>) -> Self {
        Self { repository }
    }

    pub fn default_service() -> Self {
        Self::new(Arc::new(ListeningEventRepositoryImpl::new(database())))
    }

    pub fn create_event(event: NewListeningEvent) -> Result<ListeningEvent, CustomError> {
        Self::default_service().create(event)
    }

    pub fn create(&self, event: NewListeningEvent) -> Result<ListeningEvent, CustomError> {
        self.repository.create(event)
    }

    pub fn get_by_user_and_range(
        &self,
        username: &str,
        from: Option<NaiveDateTime>,
        to: Option<NaiveDateTime>,
    ) -> Result<Vec<ListeningEvent>, CustomError> {
        self.repository.get_by_user_and_range(username, from, to)
    }

    pub fn delete_by_username(&self, username: &str) -> Result<usize, CustomError> {
        self.repository.delete_by_username(username)
    }

    pub fn delete_by_podcast_id(&self, podcast_id: i32) -> Result<usize, CustomError> {
        self.repository.delete_by_podcast_id(podcast_id)
    }
}

