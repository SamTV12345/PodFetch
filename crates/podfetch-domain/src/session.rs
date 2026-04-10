use chrono::{DateTime, NaiveDateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub username: String,
    pub session_id: String,
    pub expires: NaiveDateTime,
}

impl Session {
    pub fn new(username: String) -> Self {
        Self {
            username,
            session_id: Uuid::new_v4().to_string(),
            expires: DateTime::from_timestamp(Utc::now().timestamp() + 60 * 60 * 24, 0)
                .map(|value| value.naive_utc())
                .unwrap(),
        }
    }
}

pub trait SessionRepository: Send + Sync {
    type Error;

    fn create(&self, session: Session) -> Result<Session, Self::Error>;
    fn find_by_session_id(&self, session_id: &str) -> Result<Option<Session>, Self::Error>;
    fn delete_by_username(&self, username: &str) -> Result<usize, Self::Error>;
    fn cleanup_expired(&self, now: NaiveDateTime) -> Result<usize, Self::Error>;
}
