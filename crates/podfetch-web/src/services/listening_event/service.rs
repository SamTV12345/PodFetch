use chrono::NaiveDateTime;
use common_infrastructure::error::CustomError;
use podfetch_domain::listening_event::{
    ListeningEvent, ListeningEventRepository, NewListeningEvent,
};
use podfetch_persistence::adapters::ListeningEventRepositoryImpl;
use podfetch_persistence::db::database;
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
        user_id: i32,
        from: Option<NaiveDateTime>,
        to: Option<NaiveDateTime>,
    ) -> Result<Vec<ListeningEvent>, CustomError> {
        self.repository.get_by_user_and_range(user_id, from, to)
    }

    pub fn delete_by_user_id(&self, user_id: i32) -> Result<usize, CustomError> {
        self.repository.delete_by_user_id(user_id)
    }

    pub fn delete_by_podcast_id(&self, podcast_id: i32) -> Result<usize, CustomError> {
        self.repository.delete_by_podcast_id(podcast_id)
    }
}
