use serde::Deserialize;
use std::fmt::Display;
use utoipa::ToSchema;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Invite {
    pub id: String,
    pub role: String,
    pub created_at: chrono::NaiveDateTime,
    pub accepted_at: Option<chrono::NaiveDateTime>,
    pub explicit_consent: bool,
    pub expires_at: chrono::NaiveDateTime,
}

impl From<podfetch_domain::invite::Invite> for Invite {
    fn from(value: podfetch_domain::invite::Invite) -> Self {
        Self {
            id: value.id,
            role: value.role,
            created_at: value.created_at,
            accepted_at: value.accepted_at,
            explicit_consent: value.explicit_consent,
            expires_at: value.expires_at,
        }
    }
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct InvitePostModel {
    pub role: String,
    pub explicit_consent: bool,
}

pub trait InviteApplicationService {
    type Error;

    fn create_invite(&self, role: String, explicit_consent: bool) -> Result<Invite, Self::Error>;
    fn get_invites(&self) -> Result<Vec<Invite>, Self::Error>;
    fn get_invite(&self, invite_id: &str) -> Result<Invite, Self::Error>;
    fn get_invite_link(&self, invite_id: &str, server_url: &str) -> Result<String, Self::Error>;
    fn delete_invite(&self, invite_id: &str) -> Result<(), Self::Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum InviteControllerError<E: Display> {
    #[error("forbidden")]
    Forbidden,
    #[error("{0}")]
    Service(E),
}

pub fn create_invite<S>(
    service: &S,
    is_admin: bool,
    model: InvitePostModel,
) -> Result<Invite, InviteControllerError<S::Error>>
where
    S: InviteApplicationService,
    S::Error: Display,
{
    if !is_admin {
        return Err(InviteControllerError::Forbidden);
    }

    service
        .create_invite(model.role, model.explicit_consent)
        .map_err(InviteControllerError::Service)
}

pub fn get_invites<S>(
    service: &S,
    is_admin: bool,
) -> Result<Vec<Invite>, InviteControllerError<S::Error>>
where
    S: InviteApplicationService,
    S::Error: Display,
{
    if !is_admin {
        return Err(InviteControllerError::Forbidden);
    }

    service
        .get_invites()
        .map_err(InviteControllerError::Service)
}

pub fn get_invite<S>(
    service: &S,
    invite_id: &str,
) -> Result<Invite, InviteControllerError<S::Error>>
where
    S: InviteApplicationService,
    S::Error: Display,
{
    service
        .get_invite(invite_id)
        .map_err(InviteControllerError::Service)
}

pub fn get_invite_link<S>(
    service: &S,
    is_admin: bool,
    invite_id: &str,
    server_url: &str,
) -> Result<String, InviteControllerError<S::Error>>
where
    S: InviteApplicationService,
    S::Error: Display,
{
    if !is_admin {
        return Err(InviteControllerError::Forbidden);
    }

    service
        .get_invite_link(invite_id, server_url)
        .map_err(InviteControllerError::Service)
}

pub fn delete_invite<S>(
    service: &S,
    is_admin: bool,
    invite_id: &str,
) -> Result<(), InviteControllerError<S::Error>>
where
    S: InviteApplicationService,
    S::Error: Display,
{
    if !is_admin {
        return Err(InviteControllerError::Forbidden);
    }

    service
        .delete_invite(invite_id)
        .map_err(InviteControllerError::Service)
}
