use crate::adapters::persistence::model::user::session::SessionEntity;
use crate::adapters::persistence::repositories::user::session::SessionRepository;
use crate::domain::models::user::session::Session;
use crate::utils::error::CustomError;

pub struct SessionService;

impl SessionService {
    pub fn find_by_session_id(session_id: &str) -> Result<Session, CustomError> {
        SessionRepository::find_by_session_id(session_id)
    }

    pub fn insert_session(session: &Session) -> Result<Session, CustomError> {
        SessionRepository::insert_session(session)
    }
}