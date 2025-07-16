use crate::constants::inner_constants::{Role, ENVIRONMENT_SERVICE};
use crate::models::user::{User, UserWithAPiKey, UserWithoutPassword};
use axum::extract::Path;
use axum::{Extension, Json};
use reqwest::StatusCode;

use crate::service::user_management_service::UserManagementService;
use crate::utils::error::{CustomError, CustomErrorInner, ErrorSeverity};

use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserOnboardingModel {
    invite_id: String,
    username: String,
    password: String,
}

#[derive(Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct InvitePostModel {
    role: String,
    explicit_consent: bool,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserRoleUpdateModel {
    role: Role,
    explicit_consent: bool,
}

#[derive(Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserCoreUpdateModel {
    pub username: String,
    pub password: Option<String>,
    pub api_key: Option<String>,
}

#[utoipa::path(
post,
path="/users/",
request_body = UserOnboardingModel,
responses(
(status = 200, description = "Creates a user (admin)", body = UserWithoutPassword)),
tag="user"
)]
pub async fn onboard_user(
    Json(user_to_onboard): Json<UserOnboardingModel>,
) -> Result<Json<UserWithoutPassword>, CustomError> {
    let res = UserManagementService::onboard_user(
        user_to_onboard.username,
        user_to_onboard.password,
        user_to_onboard.invite_id,
    )?;

    Ok(Json(User::map_to_dto(res)))
}

#[utoipa::path(
get,
path="",
responses(
(status = 200, description = "Gets all users", body= Vec<UserWithoutPassword>)),
tag="info"
)]
pub async fn get_users(
    Extension(requester): Extension<User>,
) -> Result<Json<Vec<UserWithoutPassword>>, CustomError> {
    let res = UserManagementService::get_users(requester)?;

    Ok(Json(res))
}

#[utoipa::path(
get,
path = "/{username}",
responses(
(status = 200, description = "Gets a user by username", body = Option<UserWithAPiKey>)),
tag="user"
)]
pub async fn get_user(
    Path(username): Path<String>,
    Extension(requester): Extension<User>,
) -> Result<Json<UserWithAPiKey>, CustomError> {
    if username == requester.username || username == "me" {
        return Ok(Json(User::map_to_api_dto(requester)));
    }

    if !requester.is_admin() || requester.username != username {
        return Err(CustomErrorInner::Forbidden(ErrorSeverity::Warning).into());
    }

    let user = User::find_by_username(&username.clone())?;
    Ok(Json(User::map_to_api_dto(user)))
}

#[utoipa::path(
put,
path="/{username}/role",
request_body = UserRoleUpdateModel,
responses(
(status = 200, description = "Updates the role of a user", body = Option<UserWithoutPassword>)),
tag="user"
)]

pub async fn update_role(
    Path(username): Path<String>,
    Extension(requester): Extension<User>,
    Json(role): Json<UserRoleUpdateModel>,
) -> Result<Json<UserWithoutPassword>, CustomError> {
    if !requester.is_admin() {
        return Err(CustomErrorInner::Forbidden(ErrorSeverity::Warning).into());
    }
    let mut user_to_update = User::find_by_username(&username)?;

    // Update to his/her designated role
    user_to_update.role = role.role.to_string();
    user_to_update.explicit_consent = role.explicit_consent;

    let res = UserManagementService::update_user(user_to_update)?;

    Ok(Json(res))
}

#[utoipa::path(
put,
path="/{username}",
request_body=UserCoreUpdateModel,
responses(
(status = 200, description = "Creates an invite", body = UserWithAPiKey,)),
tag="user"
)]
pub async fn update_user(
    Extension(user): Extension<User>,
    Path(username): Path<String>,
    user_update: Json<UserCoreUpdateModel>,
) -> Result<Json<UserWithAPiKey>, CustomError> {
    let old_username = &user.clone().username;
    if old_username != &username {
        return Err(CustomErrorInner::Forbidden(ErrorSeverity::Warning).into());
    }
    let mut user = User::find_by_username(&username)?;

    if let Some(admin_username) = ENVIRONMENT_SERVICE.username.clone() {
        if admin_username == user.username {
            return Err(
                CustomErrorInner::Conflict("Cannot update admin user".to_string(), Info).into(),
            );
        }
    }

    if old_username != &user_update.username && !ENVIRONMENT_SERVICE.oidc_configured {
        // Check if this username is already taken
        let new_username_res = User::find_by_username(&user_update.username);
        if new_username_res.is_ok() {
            return Err(
                CustomErrorInner::Conflict("Username already taken".to_string(), Info).into(),
            );
        }
        user.username = user_update.username.to_string();
    }
    if let Some(password) = user_update.password.clone() {
        if password.trim().len() < 8 {
            return Err(CustomErrorInner::BadRequest(
                "Password must be at least 8 characters long".to_string(),
                Info,
            )
            .into());
        }
        user.password = Some(sha256::digest(password.trim()));
    }

    if let Some(api_key) = &user_update.api_key {
        user.api_key = Some(api_key.clone());
    }

    let user = User::update_user(user)?;

    Ok(Json(User::map_to_api_dto(user)))
}

use crate::models::invite::Invite;
use crate::utils::error::ErrorSeverity::{Info, Warning};

#[utoipa::path(
post,
path="/invites",
request_body=InvitePostModel,
responses(
(status = 200, description = "Creates an invite", body = Invite,)),
tag="invite"
)]
pub async fn create_invite(
    Extension(requester): Extension<User>,
    Json(invite): Json<InvitePostModel>,
) -> Result<Json<Invite>, CustomError> {
    let created_invite = UserManagementService::create_invite(
        Role::try_from(invite.role)?,
        invite.explicit_consent,
        requester,
    )?;
    Ok(Json(created_invite))
}

#[utoipa::path(
get,
path="/invites",
responses(
(status = 200, description = "Gets all invites", body = Vec<Invite>)),
tag="invite"
)]
pub async fn get_invites(
    Extension(requester): Extension<User>,
) -> Result<Json<Vec<Invite>>, CustomError> {
    if !requester.is_admin() {
        return Err(CustomErrorInner::Forbidden(Info).into());
    }

    let invites = UserManagementService::get_invites()?;

    Ok(Json(invites))
}

#[utoipa::path(
get,
path="/invites/{invite_id}",
responses(
(status = 200, description = "Gets a specific invite", body = Option<Invite>)),
tag="user"
)]
pub async fn get_invite(Path(invite_id): Path<String>) -> Result<Json<Invite>, CustomError> {
    match UserManagementService::get_invite(invite_id) {
        Ok(invite) => Ok(Json(invite)),
        Err(e) => Err(e),
    }
}

#[utoipa::path(
delete,
path="/{username}",
responses(
(status = 200, description = "Deletes a user by username")),
tag="user"
)]
pub async fn delete_user(
    Path(username): Path<String>,
    Extension(requester): Extension<User>,
) -> Result<StatusCode, CustomError> {
    if !requester.is_admin() {
        return Err(CustomErrorInner::Forbidden(Info).into());
    }

    let user_to_delete = User::find_by_username(&username)?;
    match UserManagementService::delete_user(user_to_delete) {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err(e),
    }
}

#[utoipa::path(
get,
path="/invites/{invite_id}/link",
tag="invite",
responses(
(status = 200, description = "Gets an invite by id", body = Option<String>)))]
pub async fn get_invite_link(
    Path(invite_id): Path<String>,
    requester: Extension<User>,
) -> Result<String, CustomError> {
    if !requester.is_admin() {
        return Err(CustomErrorInner::Forbidden(Warning).into());
    }

    match UserManagementService::get_invite_link(invite_id) {
        Ok(invite) => Ok(invite),
        Err(e) => Err(CustomErrorInner::BadRequest(e.to_string(), Warning).into()),
    }
}

#[utoipa::path(
delete,
path="/invites/{invite_id}",
tag="invite",
responses(
(status = 200, description = "Deletes an invite by id")))]
pub async fn delete_invite(
    invite_id: Path<String>,
    requester: Extension<User>,
) -> Result<StatusCode, CustomError> {
    if !requester.is_admin() {
        return Err(CustomErrorInner::Forbidden(Warning).into());
    }

    match UserManagementService::delete_invite(invite_id.0) {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Err(CustomErrorInner::BadRequest(e.to_string(), Warning).into()),
    }
}

pub fn get_user_router() -> OpenApiRouter {
    OpenApiRouter::new()
        .nest(
            "/users",
            OpenApiRouter::new()
                .routes(routes!(get_users))
                .routes(routes!(get_user))
                .routes(routes!(update_role))
                .routes(routes!(delete_user))
                .routes(routes!(update_user)),
        )
        .routes(routes!(delete_invite))
        .routes(routes!(get_invite_link))
        .routes(routes!(create_invite))
        .routes(routes!(get_invites))
}
