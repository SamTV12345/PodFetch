use chrono::{DateTime, NaiveDateTime, Utc};
use uuid::Uuid;

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
                .map(|v|v.naive_utc()).unwrap(),
        }
    }
}