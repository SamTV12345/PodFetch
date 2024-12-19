use crate::adapters::persistence::repositories::invite::invite::InviteRepository;
use crate::domain::models::invite::invite::Invite;
use crate::utils::error::CustomError;

pub struct InviteService;


impl InviteService {
    pub fn find_invite(id: &str) -> Result<Option<Invite>, CustomError> {
        InviteRepository::find_invite(id)
    }
}