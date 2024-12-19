use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable};
use utoipa::ToSchema;
use crate::domain::models::user::session::Session;

#[derive(Queryable, Insertable, Clone, PartialEq, Debug)]
#[table_name = "sessions"]
pub struct SessionEntity {
    pub username: String,
    pub session_id: String,
    pub expires: NaiveDateTime,
}


impl From<Session> for SessionEntity {
    fn from(session: Session) -> Self {
        Self {
            username: session.username,
            session_id: session.session_id,
            expires: session.expires,
        }
    }
}


impl Into<Session> for SessionEntity {
    fn into(self) -> Session {
        Session {
            username: self.username,
            session_id: self.session_id,
            expires: self.expires,
        }
    }
}