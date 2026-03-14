use crate::constants::inner_constants::{
    ENVIRONMENT_SERVICE, Role, STANDARD_USER, STANDARD_USER_ID,
};
use crate::models::user::{User, UserWithAPiKey, UserWithoutPassword};
use axum::extract::Path;
use axum::{Extension, Json};
use reqwest::StatusCode;

use crate::service::user_management_service::UserManagementService;
use crate::utils::error::{ApiError, CustomError, CustomErrorInner, ErrorSeverity, ErrorType};

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
) -> Result<Json<UserWithAPiKey>, ErrorType> {
    if username == requester.username || username == "me" {
        return Ok(Json(User::map_to_api_dto(requester)));
    }

    if !requester.is_admin() || requester.username != username {
        return Err(CustomErrorInner::Forbidden(Warning).into());
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
) -> Result<Json<UserWithAPiKey>, ErrorType> {
    if STANDARD_USER_ID == user.id {
        return Err(ApiError::updating_admin_not_allowed(STANDARD_USER).into());
    }

    let old_username = &user.clone().username;
    if old_username != &username {
        return Err(CustomErrorInner::Forbidden(Warning).into());
    }

    let mut user = User::find_by_username(&username)?;

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

#[cfg(test)]
mod tests {
    use crate::commands::startup::tests::handle_test_startup;
    use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
    use crate::models::invite::Invite;
    use crate::models::user::User;
    use crate::models::user::UserWithAPiKey;
    use crate::utils::error::ErrorType;
    use crate::utils::error::CustomErrorInner;
    use crate::utils::test_builder::user_test_builder::tests::UserTestDataBuilder;
    use axum::extract::Path;
    use axum::{Extension, Json};
    use serde_json::json;
    use serial_test::serial;
    use uuid::Uuid;

    fn admin_username() -> String {
        ENVIRONMENT_SERVICE
            .username
            .clone()
            .unwrap_or_else(|| "postgres".to_string())
    }

    fn unique_username(prefix: &str) -> String {
        format!("{prefix}-{}", Uuid::new_v4())
    }

    fn non_admin_user() -> User {
        UserTestDataBuilder::new().build()
    }

    fn assert_client_error_status(status: u16) {
        assert!(
            (400..500).contains(&status),
            "expected 4xx status, got {status}"
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_get_current_user_with_me_alias() {
        let server = handle_test_startup().await;

        let response = server.test_server.get("/api/v1/users/me").await;
        assert_eq!(response.status_code(), 200);

        let user = response.json::<UserWithAPiKey>();
        assert_eq!(user.username, admin_username());
        assert_eq!(user.role, "admin");
        assert!(user.read_only);
    }

    #[tokio::test]
    #[serial]
    async fn test_invite_create_get_and_delete_lifecycle() {
        let server = handle_test_startup().await;

        let create_response = server
            .test_server
            .post("/api/v1/invites")
            .json(&json!({
                "role": "user",
                "explicitConsent": true
            }))
            .await;
        assert_eq!(create_response.status_code(), 200);
        let created_invite = create_response.json::<Invite>();

        let get_single_response = server
            .test_server
            .get(&format!("/api/v1/invites/{}", created_invite.id))
            .await;
        assert_eq!(get_single_response.status_code(), 200);
        let invite = get_single_response.json::<Invite>();
        assert_eq!(invite.id, created_invite.id);
        assert_eq!(invite.role, "user");

        let list_response = server.test_server.get("/api/v1/invites").await;
        assert_eq!(list_response.status_code(), 200);
        let invites = list_response.json::<Vec<Invite>>();
        assert_eq!(invites.len(), 1);
        assert_eq!(invites[0].id, created_invite.id);

        let delete_response = server
            .test_server
            .delete(&format!("/api/v1/invites/{}", created_invite.id))
            .await;
        assert_eq!(delete_response.status_code(), 200);

        let list_after_delete = server.test_server.get("/api/v1/invites").await;
        assert_eq!(list_after_delete.status_code(), 200);
        assert!(list_after_delete.json::<Vec<Invite>>().is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn test_onboard_user_lifecycle_and_reuse_invite_conflict() {
        let server = handle_test_startup().await;
        let username = unique_username("onboard-user");
        let password = "ValidPass123";

        let create_invite_response = server
            .test_server
            .post("/api/v1/invites")
            .json(&json!({
                "role": "user",
                "explicitConsent": true
            }))
            .await;
        assert_eq!(create_invite_response.status_code(), 200);
        let invite = create_invite_response.json::<Invite>();

        let onboard_response = server
            .test_server
            .post("/api/v1/users/")
            .json(&json!({
                "inviteId": invite.id,
                "username": username,
                "password": password
            }))
            .await;
        assert_eq!(onboard_response.status_code(), 200);

        let second_onboard_response = server
            .test_server
            .post("/api/v1/users/")
            .json(&json!({
                "inviteId": invite.id,
                "username": unique_username("onboard-user-second"),
                "password": password
            }))
            .await;
        assert_eq!(second_onboard_response.status_code(), 409);
    }

    #[tokio::test]
    #[serial]
    async fn test_onboard_user_with_unknown_invite_returns_not_found() {
        let server = handle_test_startup().await;

        let response = server
            .test_server
            .post("/api/v1/users/")
            .json(&json!({
                "inviteId": "invite-does-not-exist",
                "username": unique_username("unknown-invite-user"),
                "password": "ValidPass123"
            }))
            .await;

        assert_eq!(response.status_code(), 404);
    }

    #[tokio::test]
    #[serial]
    async fn test_create_invite_rejects_invalid_role_payload() {
        let server = handle_test_startup().await;

        let response = server
            .test_server
            .post("/api/v1/invites")
            .json(&json!({
                "role": "super-admin",
                "explicitConsent": true
            }))
            .await;

        assert_client_error_status(response.status_code().as_u16());
    }

    #[tokio::test]
    #[serial]
    async fn test_invite_link_and_delete_return_bad_request_for_unknown_invite() {
        let server = handle_test_startup().await;
        let unknown_invite_id = "invite-does-not-exist";

        let link_response = server
            .test_server
            .get(&format!("/api/v1/invites/{unknown_invite_id}/link"))
            .await;
        assert_eq!(link_response.status_code(), 400);

        let delete_response = server
            .test_server
            .delete(&format!("/api/v1/invites/{unknown_invite_id}"))
            .await;
        assert_eq!(delete_response.status_code(), 400);
    }

    #[tokio::test]
    #[serial]
    async fn test_user_handlers_return_forbidden_for_non_admin_or_foreign_user() {
        let non_admin = non_admin_user();

        let get_users_result = super::get_users(Extension(non_admin.clone())).await;
        match get_users_result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden error for get_users"),
        }

        let get_invites_result = super::get_invites(Extension(non_admin.clone())).await;
        match get_invites_result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden error for get_invites"),
        }

        let invite_link_result =
            super::get_invite_link(Path("some-id".to_string()), Extension(non_admin)).await;
        match invite_link_result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden error for get_invite_link"),
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_onboard_user_rejects_weak_password() {
        let server = handle_test_startup().await;

        let create_invite_response = server
            .test_server
            .post("/api/v1/invites")
            .json(&json!({
                "role": "user",
                "explicitConsent": true
            }))
            .await;
        assert_eq!(create_invite_response.status_code(), 200);
        let invite = create_invite_response.json::<Invite>();

        let onboard_response = server
            .test_server
            .post("/api/v1/users/")
            .json(&json!({
                "inviteId": invite.id,
                "username": unique_username("weak-password-user"),
                "password": "weak"
            }))
            .await;

        assert_eq!(onboard_response.status_code(), 409);
    }

    #[tokio::test]
    #[serial]
    async fn test_user_endpoints_return_client_error_for_wrong_http_methods() {
        let server = handle_test_startup().await;

        let post_me_response = server.test_server.post("/api/v1/users/me").await;
        assert_client_error_status(post_me_response.status_code().as_u16());

        let put_invites_response = server.test_server.put("/api/v1/invites").await;
        assert_client_error_status(put_invites_response.status_code().as_u16());

        let post_invite_link_response = server
            .test_server
            .post("/api/v1/invites/some-id/link")
            .await;
        assert_client_error_status(post_invite_link_response.status_code().as_u16());
    }

    #[tokio::test]
    #[serial]
    async fn test_user_endpoints_return_not_found_for_invalid_paths() {
        let server = handle_test_startup().await;

        let wrong_user_path = server.test_server.get("/api/v1/user/me").await;
        assert_eq!(wrong_user_path.status_code(), 404);

        let wrong_invite_path = server.test_server.get("/api/v1/invite").await;
        assert_eq!(wrong_invite_path.status_code(), 404);
    }

    #[tokio::test]
    #[serial]
    async fn test_additional_user_handlers_return_forbidden_for_non_admin() {
        let non_admin = non_admin_user();

        let get_user_result = super::get_user(
            Path("other-user".to_string()),
            Extension(non_admin.clone()),
        )
        .await;
        match get_user_result {
            Err(ErrorType::CustomErrorType(err)) => {
                assert!(matches!(err.inner, CustomErrorInner::Forbidden(_)))
            }
            _ => panic!("expected forbidden error for get_user"),
        }

        let update_role_result = super::update_role(
            Path("someone".to_string()),
            Extension(non_admin.clone()),
            Json(super::UserRoleUpdateModel {
                role: crate::constants::inner_constants::Role::User,
                explicit_consent: true,
            }),
        )
        .await;
        match update_role_result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden error for update_role"),
        }

        let delete_user_result =
            super::delete_user(Path("someone".to_string()), Extension(non_admin.clone())).await;
        match delete_user_result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden error for delete_user"),
        }

        let delete_invite_result =
            super::delete_invite(Path("some-invite".to_string()), Extension(non_admin)).await;
        match delete_invite_result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden error for delete_invite"),
        }
    }
}
