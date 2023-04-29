use chrono::{NaiveDateTime, Utc};
use diesel::{Insertable, Queryable, RunQueryDsl};
use utoipa::ToSchema;
use uuid::Uuid;
use crate::schema::sessions;
use diesel::QueryDsl;
use diesel::ExpressionMethods;

#[derive(Queryable, Insertable, Clone, ToSchema, PartialEq, Debug)]
pub struct Session{
    pub username: String,
    pub session_id: String,
    pub expires: NaiveDateTime
}


impl Session{
    pub fn new(username: String) -> Self{
        Self{
            username,
            session_id: Uuid::new_v4().to_string(),
            expires: NaiveDateTime::from_timestamp_opt(chrono::Utc::now().timestamp() + 60 * 60 *
                24, 0).unwrap()
        }
    }

    pub fn insert_session(&self, conn: &mut diesel::SqliteConnection) -> Result<Self, diesel::result::Error>{
        diesel::insert_into(sessions::table)
            .values(self)
            .get_result(conn)
    }

    pub fn cleanup_sessions(conn: &mut diesel::SqliteConnection) -> Result<usize, diesel::result::Error>{
        diesel::delete(sessions::table.
            filter(sessions::expires.lt(Utc::now().naive_utc())))
            .execute(conn)
    }

    pub fn find_by_session_id(session_id: &str, conn: &mut diesel::SqliteConnection) -> Result<Self, diesel::result::Error>{
        sessions::table
            .filter(sessions::session_id.eq(session_id))
            .get_result(conn)
    }

    pub fn delete_by_username(username1: &str, conn: &mut diesel::SqliteConnection) ->
                                                                                    Result<usize, diesel::result::Error>{
        diesel::delete(sessions::table
            .filter(sessions::username.eq(username1)))
            .execute(conn)
    }
}