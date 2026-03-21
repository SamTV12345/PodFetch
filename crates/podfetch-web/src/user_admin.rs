use podfetch_domain::user_admin::{ManagedUser, UserSummary, UserWithApiKey};
use serde::Deserialize;
use std::fmt::Display;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserRoleUpdateModel {
    pub role: String,
    pub explicit_consent: bool,
}

#[derive(Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserCoreUpdateModel {
    pub username: String,
    pub password: Option<String>,
    pub api_key: Option<String>,
}

pub trait UserAdminApplicationService {
    type Error;

    fn find_by_username(&self, username: &str) -> Result<Option<ManagedUser>, Self::Error>;
    fn find_all(&self) -> Result<Vec<ManagedUser>, Self::Error>;
    fn update(&self, user: ManagedUser) -> Result<ManagedUser, Self::Error>;
    fn delete_by_username(&self, username: &str) -> Result<(), Self::Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum UserAdminControllerError<E: Display> {
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    #[error("updating admin not allowed")]
    UpdatingAdminNotAllowed,
    #[error("username already taken")]
    UsernameTaken,
    #[error("password too short")]
    PasswordTooShort,
    #[error("{0}")]
    Service(E),
}

pub fn get_users<S>(
    service: &S,
    requester_is_admin: bool,
) -> Result<Vec<UserSummary>, UserAdminControllerError<S::Error>>
where
    S: UserAdminApplicationService,
    S::Error: Display,
{
    if !requester_is_admin {
        return Err(UserAdminControllerError::Forbidden);
    }

    service
        .find_all()
        .map(|users| users.into_iter().map(|user| user.to_summary()).collect())
        .map_err(UserAdminControllerError::Service)
}

pub fn get_user<S>(
    service: &S,
    username: &str,
    requester: &ManagedUser,
    read_only_admin_id: i32,
) -> Result<UserWithApiKey, UserAdminControllerError<S::Error>>
where
    S: UserAdminApplicationService,
    S::Error: Display,
{
    if username == requester.username || username == "me" {
        return Ok(requester.to_api_dto(requester.id == read_only_admin_id));
    }

    if !requester.is_admin() || requester.username != username {
        return Err(UserAdminControllerError::Forbidden);
    }

    service
        .find_by_username(username)
        .map_err(UserAdminControllerError::Service)?
        .map(|user| user.to_api_dto(user.id == read_only_admin_id))
        .ok_or(UserAdminControllerError::NotFound)
}

pub fn update_role<S>(
    service: &S,
    username: &str,
    requester_is_admin: bool,
    role_update: UserRoleUpdateModel,
) -> Result<UserSummary, UserAdminControllerError<S::Error>>
where
    S: UserAdminApplicationService,
    S::Error: Display,
{
    if !requester_is_admin {
        return Err(UserAdminControllerError::Forbidden);
    }

    let mut user = service
        .find_by_username(username)
        .map_err(UserAdminControllerError::Service)?
        .ok_or(UserAdminControllerError::NotFound)?;
    user.role = role_update.role;
    user.explicit_consent = role_update.explicit_consent;

    service
        .update(user)
        .map(|user| user.to_summary())
        .map_err(UserAdminControllerError::Service)
}

pub fn update_user<S>(
    service: &S,
    path_username: &str,
    requester: &ManagedUser,
    user_update: UserCoreUpdateModel,
    read_only_admin_id: i32,
    oidc_configured: bool,
) -> Result<UserWithApiKey, UserAdminControllerError<S::Error>>
where
    S: UserAdminApplicationService,
    S::Error: Display,
{
    if requester.id == read_only_admin_id {
        return Err(UserAdminControllerError::UpdatingAdminNotAllowed);
    }

    if requester.username != path_username {
        return Err(UserAdminControllerError::Forbidden);
    }

    let mut user = service
        .find_by_username(path_username)
        .map_err(UserAdminControllerError::Service)?
        .ok_or(UserAdminControllerError::NotFound)?;

    if requester.username != user_update.username && !oidc_configured {
        let username_taken = service
            .find_by_username(&user_update.username)
            .map_err(UserAdminControllerError::Service)?
            .is_some();
        if username_taken {
            return Err(UserAdminControllerError::UsernameTaken);
        }
        user.username = user_update.username;
    }

    if let Some(password) = user_update.password {
        if password.trim().len() < 8 {
            return Err(UserAdminControllerError::PasswordTooShort);
        }
        user.password = Some(sha256::digest(password.trim()));
    }

    if let Some(api_key) = user_update.api_key {
        user.api_key = Some(api_key);
    }

    service
        .update(user)
        .map(|user| user.to_api_dto(user.id == read_only_admin_id))
        .map_err(UserAdminControllerError::Service)
}

pub fn delete_user<S>(
    service: &S,
    requester_is_admin: bool,
    username: &str,
) -> Result<(), UserAdminControllerError<S::Error>>
where
    S: UserAdminApplicationService,
    S::Error: Display,
{
    if !requester_is_admin {
        return Err(UserAdminControllerError::Forbidden);
    }

    service
        .find_by_username(username)
        .map_err(UserAdminControllerError::Service)?
        .ok_or(UserAdminControllerError::NotFound)?;

    service
        .delete_by_username(username)
        .map_err(UserAdminControllerError::Service)
}
