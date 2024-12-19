use chrono::NaiveDateTime;
use crate::constants::inner_constants::Role;

pub struct Invite {
    pub id: String,
    pub role: Role,
    pub created_at: NaiveDateTime,
    pub accepted_at: Option<NaiveDateTime>,
    pub explicit_consent: bool,
    pub expires_at: NaiveDateTime,
}