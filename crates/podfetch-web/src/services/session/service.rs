use chrono::{NaiveDateTime, Utc};
use common_infrastructure::error::CustomError;
use podfetch_domain::session::{Session, SessionRepository};
use podfetch_persistence::adapters::SessionRepositoryImpl;
use podfetch_persistence::db::database;
use std::sync::Arc;

#[derive(Clone)]
pub struct SessionService {
    repository: Arc<dyn SessionRepository<Error = CustomError>>,
}

impl SessionService {
    pub fn new(repository: Arc<dyn SessionRepository<Error = CustomError>>) -> Self {
        Self { repository }
    }

    pub fn default_service() -> Self {
        Self::new(Arc::new(SessionRepositoryImpl::new(database())))
    }

    pub fn create_session(&self, username: String) -> Result<Session, CustomError> {
        self.repository.create(Session::new(username))
    }

    pub fn create_existing_session(&self, session: Session) -> Result<Session, CustomError> {
        self.repository.create(session)
    }

    pub fn find_by_session_id(&self, session_id: &str) -> Result<Option<Session>, CustomError> {
        self.repository.find_by_session_id(session_id)
    }

    pub fn delete_by_username(&self, username: &str) -> Result<usize, CustomError> {
        self.repository.delete_by_username(username)
    }

    pub fn cleanup_expired(&self) -> Result<usize, CustomError> {
        self.repository.cleanup_expired(Utc::now().naive_utc())
    }

    pub fn cleanup_expired_at(&self, now: NaiveDateTime) -> Result<usize, CustomError> {
        self.repository.cleanup_expired(now)
    }
}
