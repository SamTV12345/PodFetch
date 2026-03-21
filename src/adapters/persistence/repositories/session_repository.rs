use crate::utils::error::CustomError;
use chrono::NaiveDateTime;
use podfetch_domain::session::{Session, SessionRepository};
use podfetch_persistence::db::Database;
use podfetch_persistence::session::DieselSessionRepository;

pub struct SessionRepositoryImpl {
    inner: DieselSessionRepository,
}

impl SessionRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselSessionRepository::new(database),
        }
    }
}

impl SessionRepository for SessionRepositoryImpl {
    type Error = CustomError;

    fn create(&self, session: Session) -> Result<Session, Self::Error> {
        self.inner.create(session).map_err(Into::into)
    }

    fn find_by_session_id(&self, session_id: &str) -> Result<Option<Session>, Self::Error> {
        self.inner
            .find_by_session_id(session_id)
            .map_err(Into::into)
    }

    fn delete_by_username(&self, username: &str) -> Result<usize, Self::Error> {
        self.inner.delete_by_username(username).map_err(Into::into)
    }

    fn cleanup_expired(&self, now: NaiveDateTime) -> Result<usize, Self::Error> {
        self.inner.cleanup_expired(now).map_err(Into::into)
    }
}
