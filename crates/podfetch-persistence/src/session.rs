use crate::db::{Database, PersistenceError};
use chrono::NaiveDateTime;
use diesel::prelude::{Insertable, Queryable};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use podfetch_domain::session::{Session, SessionRepository};

diesel::table! {
    sessions (session_id) {
        username -> Text,
        session_id -> Text,
        expires -> Timestamp,
    }
}

#[derive(Queryable, Insertable, Clone)]
#[diesel(table_name = sessions)]
struct SessionEntity {
    username: String,
    session_id: String,
    expires: NaiveDateTime,
}

impl From<SessionEntity> for Session {
    fn from(value: SessionEntity) -> Self {
        Self {
            username: value.username,
            session_id: value.session_id,
            expires: value.expires,
        }
    }
}

impl From<Session> for SessionEntity {
    fn from(value: Session) -> Self {
        Self {
            username: value.username,
            session_id: value.session_id,
            expires: value.expires,
        }
    }
}

pub struct DieselSessionRepository {
    database: Database,
}

impl DieselSessionRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

impl SessionRepository for DieselSessionRepository {
    type Error = PersistenceError;

    fn create(&self, session: Session) -> Result<Session, Self::Error> {
        use self::sessions::dsl::*;

        diesel::insert_into(sessions)
            .values(SessionEntity::from(session))
            .get_result::<SessionEntity>(&mut self.database.connection()?)
            .map(Into::into)
            .map_err(Into::into)
    }

    fn find_by_session_id(&self, session_to_find: &str) -> Result<Option<Session>, Self::Error> {
        use self::sessions::dsl::*;

        sessions
            .filter(session_id.eq(session_to_find))
            .first::<SessionEntity>(&mut self.database.connection()?)
            .optional()
            .map(|session| session.map(Into::into))
            .map_err(Into::into)
    }

    fn delete_by_username(&self, username_to_delete: &str) -> Result<usize, Self::Error> {
        use self::sessions::dsl::*;

        diesel::delete(sessions.filter(username.eq(username_to_delete)))
            .execute(&mut self.database.connection()?)
            .map_err(Into::into)
    }

    fn cleanup_expired(&self, now: NaiveDateTime) -> Result<usize, Self::Error> {
        use self::sessions::dsl::*;

        diesel::delete(sessions.filter(expires.lt(now)))
            .execute(&mut self.database.connection()?)
            .map_err(Into::into)
    }
}
