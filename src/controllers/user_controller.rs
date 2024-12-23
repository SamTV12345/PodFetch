use crate::constants::inner_constants::{Role, ENVIRONMENT_SERVICE};
use crate::models::user::User;

use crate::service::user_management_service::UserManagementService;
use crate::utils::error::CustomError;
use actix_web::web::{Json, Path};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};

use utoipa::ToSchema;

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
    role: Role,
    explicit_consent: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserRoleUpdateModel {
    role: Role,
    explicit_consent: bool,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserCoreUpdateModel {
    pub username: String,
    pub password: Option<String>,
    pub api_key: Option<String>,
}

#[utoipa::path(
context_path="/api/v1",
request_body = UserOnboardingModel,
responses(
(status = 200, description = "Creates a user (admin)")),
tag="info"
)]
#[post("/users/")]
pub async fn onboard_user(
    user_onboarding: web::Json<UserOnboardingModel>,
) -> Result<HttpResponse, CustomError> {
    let user_to_onboard = user_onboarding.into_inner();

    let res = UserManagementService::onboard_user(
        user_to_onboard.username,
        user_to_onboard.password,
        user_to_onboard.invite_id,
    )?;

    Ok(HttpResponse::Ok().json(User::map_to_dto(res)))
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets all users", body= Vec<UserOnboardingModel>)),
tag="info"
)]
#[get("")]
pub async fn get_users(requester: Option<web::ReqData<User>>) -> Result<HttpResponse, CustomError> {
    let res = UserManagementService::get_users(requester.unwrap().into_inner())?;

    Ok(HttpResponse::Ok().json(res))
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets a user by username", body = Option<User>)),
tag="info"
)]
#[get("/{username}")]
pub async fn get_user(
    username: Path<String>,
    requester: Option<web::ReqData<User>>,
) -> Result<HttpResponse, CustomError> {
    let user = requester.unwrap().into_inner();
    let username = username.into_inner();
    if user.username == username || username == "me" {
        return Ok(HttpResponse::Ok().json(User::map_to_api_dto(user)));
    }

    if !user.is_admin() || user.username != username {
        return Err(CustomError::Forbidden);
    }

    let user = User::find_by_username(&username.clone())?;
    Ok(HttpResponse::Ok().json(User::map_to_api_dto(user)))
}

#[utoipa::path(
context_path="/api/v1",
request_body = UserOnboardingModel,
responses(
(status = 200, description = "Updates the role of a user", body = Option<User>)),
tag="info"
)]
#[put("/{username}/role")]
pub async fn update_role(
    role: web::Json<UserRoleUpdateModel>,
    username: web::Path<String>,
    requester: Option<web::ReqData<User>>,
) -> Result<HttpResponse, CustomError> {
    if !requester.unwrap().is_admin() {
        return Err(CustomError::Forbidden);
    }
    let mut user_to_update = User::find_by_username(&username)?;

    // Update to his/her designated role
    user_to_update.role = role.role.to_string();
    user_to_update.explicit_consent = role.explicit_consent;

    let res = UserManagementService::update_user(user_to_update)?;

    Ok(HttpResponse::Ok().json(res))
}

#[put("/{username}")]
pub async fn update_user(
    user: Option<web::ReqData<User>>,
    username: Path<String>,
    user_update: Json<UserCoreUpdateModel>,
) -> Result<HttpResponse, CustomError> {
    let username = username.into_inner();
    let old_username = &user.clone().unwrap().username;
    if user.is_none() {
        return Err(CustomError::Forbidden);
    }
    if old_username != &username {
        return Err(CustomError::Forbidden);
    }
    let mut user = User::find_by_username(&username)?;

    if let Some(admin_username) = ENVIRONMENT_SERVICE.get().unwrap().username.clone() {
        if admin_username == user.username {
            return Err(CustomError::Conflict(
                "Cannot update admin user".to_string(),
            ));
        }
    }

    if old_username != &user_update.username && !ENVIRONMENT_SERVICE.get().unwrap().oidc_configured
    {
        // Check if this username is already taken
        let new_username_res = User::find_by_username(&user_update.username);
        if new_username_res.is_ok() {
            return Err(CustomError::Conflict("Username already taken".to_string()));
        }
        user.username = user_update.username.to_string();
    }
    if let Some(password) = user_update.password.clone() {
        if password.trim().len() < 8 {
            return Err(CustomError::BadRequest(
                "Password must be at least 8 characters long".to_string(),
            ));
        }
        user.password = Some(sha256::digest(password.trim()));
    }

    if let Some(api_key) = user_update.into_inner().api_key {
        user.api_key = Some(api_key);
    }

    let user = User::update_user(user)?;

    Ok(HttpResponse::Ok().json(User::map_to_api_dto(user)))
}

use crate::models::invite::Invite;
#[utoipa::path(
context_path="/api/v1",
request_body=InvitePostModel,
responses(
(status = 200, description = "Creates an invite", body = Invite,)),
tag="info"
)]
#[post("/invites")]
pub async fn create_invite(
    invite: web::Json<InvitePostModel>,
    requester: Option<web::ReqData<User>>,
) -> impl Responder {
    let invite = invite.into_inner();

    let created_invite = UserManagementService::create_invite(
        invite.role,
        invite.explicit_consent,
        requester.unwrap().into_inner(),
    )
    .expect("Error creating invite");
    HttpResponse::Ok().json(created_invite)
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets all invites", body = Vec<Invite>)),
tag="info"
)]
#[get("/invites")]
pub async fn get_invites(
    requester: Option<web::ReqData<User>>,
) -> Result<HttpResponse, CustomError> {
    if !requester.unwrap().is_admin() {
        return Err(CustomError::Forbidden);
    }

    let invites = UserManagementService::get_invites()?;

    Ok(HttpResponse::Ok().json(invites))
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets a specific invite", body = Option<Invite>)),
tag="info"
)]
#[get("/users/invites/{invite_id}")]
pub async fn get_invite(invite_id: web::Path<String>) -> Result<HttpResponse, CustomError> {
    match UserManagementService::get_invite(invite_id.into_inner()) {
        Ok(invite) => Ok(HttpResponse::Ok().json(invite)),
        Err(e) => Ok(HttpResponse::BadRequest().body(e.to_string())),
    }
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Deletes a user by username")),
tag="info"
)]
#[delete("/{username}")]
pub async fn delete_user(
    username: Path<String>,
    requester: Option<web::ReqData<User>>,
) -> Result<HttpResponse, CustomError> {
    if !requester.unwrap().is_admin() {
        return Err(CustomError::Forbidden);
    }

    let user_to_delete = User::find_by_username(&username).unwrap();
    match UserManagementService::delete_user(user_to_delete) {
        Ok(_) => Ok(HttpResponse::Ok().into()),
        Err(e) => Err(e),
    }
}

#[utoipa::path(
context_path="/api/v1",
tag="info",
responses(
(status = 200, description = "Gets an invite by id", body = Option<Invite>)))]
#[get("/invites/{invite_id}/link")]
pub async fn get_invite_link(
    invite_id: Path<String>,
    requester: Option<web::ReqData<User>>,
) -> impl Responder {
    if !requester.unwrap().is_admin() {
        return HttpResponse::Forbidden().body("You are not authorized to perform this action");
    }

    match UserManagementService::get_invite_link(
        invite_id.into_inner(),
        ENVIRONMENT_SERVICE.get().unwrap(),
    ) {
        Ok(invite) => HttpResponse::Ok().json(invite),
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}

#[utoipa::path(
context_path="/api/v1",
tag="info",
responses(
(status = 200, description = "Deletes an invite by id")))]
#[delete("/invites/{invite_id}")]
pub async fn delete_invite(
    invite_id: web::Path<String>,
    requester: Option<web::ReqData<User>>,
) -> impl Responder {
    if !requester.unwrap().is_admin() {
        return HttpResponse::Forbidden().body("You are not authorized to perform this action");
    }

    match UserManagementService::delete_invite(invite_id.into_inner()) {
        Ok(_) => HttpResponse::Ok().into(),
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}
