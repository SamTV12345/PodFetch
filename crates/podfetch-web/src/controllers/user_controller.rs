use crate::app_state::AppState;
use crate::invite::Invite;
use crate::invite::{self, InviteControllerError, InvitePostModel};
use crate::role::STANDARD_USER;
use crate::user_admin::{
    self, UserAdminControllerError, UserCoreUpdateModel, UserRoleUpdateModel, UserSummary,
    UserWithApiKey,
};
use crate::user_onboarding::{self, UserOnboardingModel};
use axum::extract::{Path, State};
use axum::{Extension, Json};
use podfetch_domain::user::User;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::services::user_admin::service::map_requester;
use common_infrastructure::error::{
    ApiError, CustomError, CustomErrorInner, ErrorSeverity, ErrorType,
};

use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[utoipa::path(
post,
path="/users/",
request_body = UserOnboardingModel,
responses(
(status = 200, description = "Creates a user (admin)", body = UserSummary)),
tag="user"
)]
pub async fn onboard_user(
    State(state): State<AppState>,
    Json(user_to_onboard): Json<UserOnboardingModel>,
) -> Result<Json<UserSummary>, CustomError> {
    user_onboarding::onboard_user(state.user_onboarding_service.as_ref(), user_to_onboard).map(Json)
}

#[utoipa::path(
get,
path="",
responses(
(status = 200, description = "Gets all users", body= Vec<UserSummary>)),
tag="info"
)]
pub async fn get_users(
    State(state): State<AppState>,
    Extension(requester): Extension<User>,
) -> Result<Json<Vec<UserSummary>>, CustomError> {
    user_admin::get_users(state.user_admin_service.as_ref(), requester.is_admin())
        .map(Json)
        .map_err(map_user_admin_error)
}

#[utoipa::path(
get,
path = "/{username}",
responses(
(status = 200, description = "Gets a user by username", body = Option<UserWithApiKey>)),
tag="user"
)]
pub async fn get_user(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Extension(requester): Extension<User>,
) -> Result<Json<UserWithApiKey>, ErrorType> {
    user_admin::get_user(
        state.user_admin_service.as_ref(),
        &username,
        &map_requester(&requester),
        state.user_admin_service.read_only_admin_id(),
    )
    .map(Json)
    .map_err(map_user_admin_error_type)
}

#[utoipa::path(
put,
path="/{username}/role",
request_body = UserRoleUpdateModel,
responses(
(status = 200, description = "Updates the role of a user", body = Option<UserSummary>)),
tag="user"
)]

pub async fn update_role(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Extension(requester): Extension<User>,
    Json(role): Json<UserRoleUpdateModel>,
) -> Result<Json<UserSummary>, CustomError> {
    user_admin::update_role(
        state.user_admin_service.as_ref(),
        &username,
        requester.is_admin(),
        UserRoleUpdateModel {
            role: role.role.to_string(),
            explicit_consent: role.explicit_consent,
        },
    )
    .map(Json)
    .map_err(map_user_admin_error)
}

#[utoipa::path(
put,
path="/{username}",
request_body=UserCoreUpdateModel,
responses(
(status = 200, description = "Creates an invite", body = UserWithApiKey,)),
tag="user"
)]
pub async fn update_user(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(username): Path<String>,
    user_update: Json<UserCoreUpdateModel>,
) -> Result<Json<UserWithApiKey>, ErrorType> {
    user_admin::update_user(
        state.user_admin_service.as_ref(),
        &username,
        &map_requester(&user),
        user_update.0,
        state.user_admin_service.read_only_admin_id(),
        state.user_admin_service.oidc_configured(),
    )
    .map(Json)
    .map_err(map_user_admin_error_type)
}

use common_infrastructure::error::ErrorSeverity::{Info, Warning};

#[utoipa::path(
post,
path="/invites",
request_body=InvitePostModel,
responses(
(status = 200, description = "Creates an invite", body = Invite,)),
tag="invite"
)]
pub async fn create_invite(
    State(state): State<AppState>,
    Extension(requester): Extension<User>,
    Json(invite): Json<InvitePostModel>,
) -> Result<Json<Invite>, CustomError> {
    invite::create_invite(state.invite_service.as_ref(), requester.is_admin(), invite)
        .map(Json)
        .map_err(map_invite_controller_error)
}

#[utoipa::path(
get,
path="/invites",
responses(
(status = 200, description = "Gets all invites", body = Vec<Invite>)),
tag="invite"
)]
pub async fn get_invites(
    State(state): State<AppState>,
    Extension(requester): Extension<User>,
) -> Result<Json<Vec<Invite>>, CustomError> {
    invite::get_invites(state.invite_service.as_ref(), requester.is_admin())
        .map(Json)
        .map_err(map_invite_controller_error)
}

#[utoipa::path(
get,
path="/invites/{invite_id}",
responses(
(status = 200, description = "Gets a specific invite", body = Option<Invite>)),
tag="user"
)]
pub async fn get_invite(
    State(state): State<AppState>,
    Path(invite_id): Path<String>,
) -> Result<Json<Invite>, CustomError> {
    invite::get_invite(state.invite_service.as_ref(), &invite_id)
        .map(Json)
        .map_err(map_invite_controller_error)
}

#[utoipa::path(
delete,
path="/{username}",
responses(
(status = 200, description = "Deletes a user by username")),
tag="user"
)]
pub async fn delete_user(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Extension(requester): Extension<User>,
) -> Result<StatusCode, CustomError> {
    user_admin::delete_user(
        state.user_admin_service.as_ref(),
        requester.is_admin(),
        &username,
    )
    .map(|_| StatusCode::OK)
    .map_err(map_user_admin_error)
}

#[utoipa::path(
get,
path="/invites/{invite_id}/link",
tag="invite",
responses(
(status = 200, description = "Gets an invite by id", body = Option<String>)))]
pub async fn get_invite_link(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Path(invite_id): Path<String>,
    requester: Extension<User>,
) -> Result<String, CustomError> {
    let server_url = crate::url_rewriting::resolve_server_url_from_headers(&headers);
    invite::get_invite_link(
        state.invite_service.as_ref(),
        requester.is_admin(),
        &invite_id,
        &server_url,
    )
    .map_err(map_invite_link_error)
}

#[utoipa::path(
delete,
path="/invites/{invite_id}",
tag="invite",
responses(
(status = 200, description = "Deletes an invite by id")))]
pub async fn delete_invite(
    State(state): State<AppState>,
    invite_id: Path<String>,
    requester: Extension<User>,
) -> Result<StatusCode, CustomError> {
    invite::delete_invite(
        state.invite_service.as_ref(),
        requester.is_admin(),
        &invite_id.0,
    )
    .map(|_| StatusCode::OK)
    .map_err(map_invite_link_error)
}

fn map_invite_controller_error(error: InviteControllerError<CustomError>) -> CustomError {
    match error {
        InviteControllerError::Forbidden => CustomErrorInner::Forbidden(Warning).into(),
        InviteControllerError::Service(error) => error,
    }
}

fn map_invite_link_error(error: InviteControllerError<CustomError>) -> CustomError {
    match error {
        InviteControllerError::Forbidden => CustomErrorInner::Forbidden(Warning).into(),
        InviteControllerError::Service(error) => {
            CustomErrorInner::BadRequest(error.to_string(), Warning).into()
        }
    }
}

fn map_user_admin_error(error: UserAdminControllerError<CustomError>) -> CustomError {
    match error {
        UserAdminControllerError::Forbidden => {
            CustomErrorInner::Forbidden(ErrorSeverity::Warning).into()
        }
        UserAdminControllerError::NotFound => CustomErrorInner::NotFound(Info).into(),
        UserAdminControllerError::UpdatingAdminNotAllowed => {
            CustomErrorInner::BadRequest("UPDATE_OF_ADMIN_NOT_ALLOWED".to_string(), Info).into()
        }
        UserAdminControllerError::UsernameTaken => {
            CustomErrorInner::Conflict("Username already taken".to_string(), Info).into()
        }
        UserAdminControllerError::PasswordTooShort => CustomErrorInner::BadRequest(
            "Password must be at least 8 characters long".to_string(),
            Info,
        )
        .into(),
        UserAdminControllerError::Service(error) => error,
    }
}

fn map_user_admin_error_type(error: UserAdminControllerError<CustomError>) -> ErrorType {
    match error {
        UserAdminControllerError::UpdatingAdminNotAllowed => {
            ApiError::updating_admin_not_allowed(STANDARD_USER).into()
        }
        other => map_user_admin_error(other).into(),
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserLocaleUpdate {
    pub country: Option<String>,
    pub language: Option<String>,
}

#[utoipa::path(
    put,
    path = "/me/locale",
    request_body = UserLocaleUpdate,
    responses((status = 200, description = "Updates the authenticated user's country/language preference", body = UserWithApiKey)),
    tag = "user"
)]
pub async fn update_locale(
    State(state): State<AppState>,
    Extension(requester): Extension<User>,
    Json(update): Json<UserLocaleUpdate>,
) -> Result<Json<UserWithApiKey>, CustomError> {
    let read_only_admin_id = state.user_admin_service.read_only_admin_id();

    let mut user = requester.clone();
    user.country = sanitize_locale(update.country.as_deref(), 8);
    user.language = sanitize_locale(update.language.as_deref(), 8);
    let updated = state.user_admin_service.update_user(user)?;
    let read_only = updated.id == read_only_admin_id;
    Ok(Json(updated.to_api_dto(read_only).into()))
}

fn sanitize_locale(value: Option<&str>, max_len: usize) -> Option<String> {
    value
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.chars().take(max_len).collect::<String>())
}

pub fn get_user_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .nest(
            "/users",
            OpenApiRouter::new()
                .routes(routes!(get_users))
                .routes(routes!(get_user))
                .routes(routes!(update_role))
                .routes(routes!(update_locale))
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
    use crate::app_state::AppState;
    use crate::invite::Invite;
    use crate::test_support::tests::handle_test_startup;
    use crate::test_utils::test_builder::user_test_builder::tests::UserTestDataBuilder;
    use crate::user_admin::UserWithApiKey;
    use axum::extract::{Path, State};
    use axum::{Extension, Json};
    use common_infrastructure::error::{CustomErrorInner, ErrorType};
    use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
    use podfetch_domain::user::User;
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

    fn app_state() -> AppState {
        AppState::new()
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

        let user = response.json::<UserWithApiKey>();
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

        let get_users_result =
            super::get_users(State(app_state()), Extension(non_admin.clone())).await;
        match get_users_result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden error for get_users"),
        }

        let get_invites_result =
            super::get_invites(State(app_state()), Extension(non_admin.clone())).await;
        match get_invites_result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden error for get_invites"),
        }

        let invite_link_result = super::get_invite_link(
            State(app_state()),
            axum::http::HeaderMap::new(),
            Path("some-id".to_string()),
            Extension(non_admin),
        )
        .await;
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
            State(app_state()),
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
            State(app_state()),
            Path("someone".to_string()),
            Extension(non_admin.clone()),
            Json(super::UserRoleUpdateModel {
                role: "user".to_string(),
                explicit_consent: true,
            }),
        )
        .await;
        match update_role_result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden error for update_role"),
        }

        let delete_user_result = super::delete_user(
            State(app_state()),
            Path("someone".to_string()),
            Extension(non_admin.clone()),
        )
        .await;
        match delete_user_result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden error for delete_user"),
        }

        let delete_invite_result = super::delete_invite(
            State(app_state()),
            Path("some-invite".to_string()),
            Extension(non_admin),
        )
        .await;
        match delete_invite_result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden error for delete_invite"),
        }
    }
}
