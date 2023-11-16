use crate::dbconfig::schema::sessions;
use crate::{execute_with_conn, DBType as DbConnection};
use chrono::{NaiveDateTime, Utc};
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::{Insertable, Queryable, RunQueryDsl};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Queryable, Insertable, Clone, ToSchema, PartialEq, Debug)]
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
            expires: NaiveDateTime::from_timestamp_opt(Utc::now().timestamp() + 60 * 60 * 24, 0)
                .unwrap(),
        }
    }

    #[allow(clippy::redundant_closure_call)]
    pub fn insert_session(&self, conn: &mut DbConnection) -> Result<Self, diesel::result::Error> {
        execute_with_conn!(conn, |conn| diesel::insert_into(sessions::table)
            .values(self)
            .get_result(conn));
    }

    pub fn cleanup_sessions(conn: &mut DbConnection) -> Result<usize, diesel::result::Error> {
        diesel::delete(sessions::table.filter(sessions::expires.lt(Utc::now().naive_utc())))
            .execute(conn)
    }

    pub fn find_by_session_id(
        session_id: &str,
        conn: &mut DbConnection,
    ) -> Result<Self, diesel::result::Error> {
        sessions::table
            .filter(sessions::session_id.eq(session_id))
            .get_result(conn)
    }

    pub fn delete_by_username(
        username1: &str,
        conn: &mut DbConnection,
    ) -> Result<usize, diesel::result::Error> {
        diesel::delete(sessions::table.filter(sessions::username.eq(username1))).execute(conn)
    }
}
