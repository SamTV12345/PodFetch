use crate::app_state::AppState;
use crate::audiobookshelf_api::auth_middleware::AuthenticatedUser;
use crate::audiobookshelf_api::dto::media_progress::MediaProgressDto;
use crate::audiobookshelf_api::dto::user::{AbsUserDto, LoginResponse, ServerSettingsDto};
use axum::Json;
use axum::extract::State;
use common_infrastructure::error::CustomError;
use podfetch_domain::audiobookshelf::library::MediaType;
use serde::Deserialize;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[derive(Deserialize, utoipa::ToSchema)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
}

#[utoipa::path(
    post,
    path = "/login",
    request_body = LoginPayload,
    responses(
        (status = 200, description = "Authenticated", body = LoginResponse),
        (status = 401, description = "Invalid credentials")
    ),
    tag = "audiobookshelf"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<LoginResponse>, CustomError> {
    let user = state
        .audiobookshelf_login_service
        .authenticate(&payload.username, &payload.password)?;

    let progress = state
        .audiobookshelf_media_progress_service
        .list_for_user(user.id)
        .unwrap_or_default();
    let progress_dtos: Vec<MediaProgressDto> =
        progress.iter().map(MediaProgressDto::from).collect();

    let default_lib_id = state
        .audiobookshelf_library_service
        .find_default_podcasts_library()?
        .map(|l| l.id);

    Ok(Json(LoginResponse {
        user: AbsUserDto::from_user(&user, progress_dtos),
        user_default_library_id: default_lib_id,
        server_settings: ServerSettingsDto::default_settings(),
        ereader_devices: Vec::new(),
        source: "podfetch".to_string(),
    }))
}

#[utoipa::path(
    post,
    path = "/api/authorize",
    responses(
        (status = 200, description = "Bearer token is valid", body = LoginResponse),
        (status = 401, description = "Invalid token")
    ),
    tag = "audiobookshelf"
)]
pub async fn authorize(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<LoginResponse>, CustomError> {
    let progress = state
        .audiobookshelf_media_progress_service
        .list_for_user(user.id)
        .unwrap_or_default();
    let progress_dtos: Vec<MediaProgressDto> =
        progress.iter().map(MediaProgressDto::from).collect();

    let default_lib_id = state
        .audiobookshelf_library_service
        .find_default_podcasts_library()
        .ok()
        .flatten()
        .or_else(|| {
            state
                .audiobookshelf_library_service
                .list()
                .ok()
                .and_then(|l| l.into_iter().next())
        })
        .map(|l| l.id);

    let _ = MediaType::Podcast; // silence dead-import lint if mediaType later unused
    Ok(Json(LoginResponse {
        user: AbsUserDto::from_user(&user, progress_dtos),
        user_default_library_id: default_lib_id,
        server_settings: ServerSettingsDto::default_settings(),
        ereader_devices: Vec::new(),
        source: "podfetch".to_string(),
    }))
}

#[utoipa::path(
    post,
    path = "/logout",
    responses((status = 200, description = "Logged out")),
    tag = "audiobookshelf"
)]
pub async fn logout(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<serde_json::Value>, CustomError> {
    if state.environment.audiobookshelf_rotate_api_key_on_logout {
        state.audiobookshelf_login_service.rotate_api_key(user)?;
    }
    Ok(Json(serde_json::json!({ "success": true })))
}

pub fn get_auth_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(login))
}

pub fn get_authorize_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(authorize))
        .routes(routes!(logout))
}
