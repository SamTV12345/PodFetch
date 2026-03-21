use podfetch_domain::invite::Invite;
use serde::Deserialize;
use std::fmt::Display;
use utoipa::ToSchema;

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
    fn get_invite_link(&self, invite_id: &str) -> Result<String, Self::Error>;
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
) -> Result<String, InviteControllerError<S::Error>>
where
    S: InviteApplicationService,
    S::Error: Display,
{
    if !is_admin {
        return Err(InviteControllerError::Forbidden);
    }

    service
        .get_invite_link(invite_id)
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
