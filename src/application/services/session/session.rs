use crate::adapters::persistence::repositories::user::session::SessionRepository;

pub struct SessionService;

impl SessionService {
    pub fn find_by_session_id() {
        SessionRepository::find_by_session_id()
    }
}