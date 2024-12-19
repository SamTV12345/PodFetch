use crate::constants::inner_constants::Role;
use crate::adapters::persistence::dbconfig::schema::invites;
use crate::utils::error::{map_db_error, CustomError};
use chrono::NaiveDateTime;
use diesel::associations::HasTable;
use diesel::ExpressionMethods;
use diesel::{Identifiable, Insertable,Queryable};
use utoipa::ToSchema;
use crate::domain::models::invite::invite::Invite;

#[derive(Queryable, Insertable, Identifiable, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct InviteEntity {
    pub id: String,
    pub role: String,
    pub created_at: NaiveDateTime,
    pub accepted_at: Option<NaiveDateTime>,
    pub explicit_consent: bool,
    pub expires_at: NaiveDateTime,
}

impl From<Invite> for InviteEntity {
    fn from(value: Invite) -> Self {
        InviteEntity {
            id: value.id,
            role: value.role.to_string(),
            created_at: value.created_at,
            accepted_at: value.accepted_at,
            explicit_consent: value.explicit_consent,
            expires_at: value.expires_at,
        }
    }
}


impl Into<Invite> for InviteEntity {
    fn into(self) -> Invite {
        Invite {
            id: self.id,
            role: Role::from(&self.role),
            created_at: self.created_at,
            accepted_at: self.accepted_at,
            explicit_consent: self.explicit_consent,
            expires_at: self.expires_at,
        }
    }
}