use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Invite {
    pub id: String,
    pub role: String,
    pub created_at: NaiveDateTime,
    pub accepted_at: Option<NaiveDateTime>,
    pub explicit_consent: bool,
    pub expires_at: NaiveDateTime,
}

pub trait InviteRepository: Send + Sync {
    type Error;

    fn create(&self, role: &str, explicit_consent: bool) -> Result<Invite, Self::Error>;
    fn find_by_id(&self, invite_id: &str) -> Result<Option<Invite>, Self::Error>;
    fn find_all(&self) -> Result<Vec<Invite>, Self::Error>;
    fn invalidate(&self, invite_id: &str) -> Result<(), Self::Error>;
    fn delete(&self, invite_id: &str) -> Result<(), Self::Error>;
}
