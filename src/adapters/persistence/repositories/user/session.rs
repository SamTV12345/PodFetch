use chrono::Utc;
use diesel::RunQueryDsl;
use futures_util::StreamExt;
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::adapters::persistence::dbconfig::schema::sessions;
use crate::adapters::persistence::model::user::session::SessionEntity;
use crate::domain::models::user::session::Session;
use crate::execute_with_conn;
use crate::utils::error::{map_db_error, CustomError};

pub struct SessionRepository;

use diesel::QueryDsl;
use diesel::ExpressionMethods;
impl SessionRepository {
    #[allow(clippy::redundant_closure_call)]
    pub fn insert_session(session: &Session) -> Result<Session, CustomError> {
        let session_entity: SessionEntity = session.into();

        execute_with_conn!(|conn| diesel::insert_into(sessions::table)
            .values(session_entity)
            .get_result(conn)
            .map_err(map_db_error)
            .map(|s: SessionEntity| s.into()))

    }

    pub fn cleanup_sessions(conn: &mut crate::adapters::persistence::dbconfig::DBType) -> Result<usize, diesel::result::Error> {
        diesel::delete(sessions::table.filter(sessions::expires.lt(Utc::now().naive_utc())))
            .execute(conn)
    }

    pub fn find_by_session_id(
        session_id: &str
    ) -> Result<Session, CustomError> {
        sessions::table
            .filter(sessions::session_id.eq(session_id))
            .get_result::<SessionEntity>(&mut get_connection())
            .map_err(map_db_error)
            .map(|s| s.into())
    }

    pub fn delete_by_username(
        username1: &str
    ) -> Result<usize, diesel::result::Error> {
        diesel::delete(sessions::table.filter(sessions::username.eq(username1))).execute(&mut get_connection())
    }
}