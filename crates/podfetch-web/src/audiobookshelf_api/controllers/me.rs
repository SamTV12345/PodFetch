use crate::app_state::AppState;
use crate::audiobookshelf_api::auth_middleware::AuthenticatedUser;
use crate::audiobookshelf_api::dto::media_progress::MediaProgressDto;
use crate::audiobookshelf_api::dto::user::AbsUserDto;
use axum::Json;
use axum::extract::State;
use common_infrastructure::error::CustomError;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[utoipa::path(
    get,
    path = "/api/me",
    responses((status = 200, description = "Current user", body = AbsUserDto)),
    tag = "audiobookshelf"
)]
pub async fn get_me(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
) -> Result<Json<AbsUserDto>, CustomError> {
    let progress = state
        .audiobookshelf_media_progress_service
        .list_for_user(user.id)
        .unwrap_or_default();
    let progress_dtos: Vec<MediaProgressDto> = progress.iter().map(MediaProgressDto::from).collect();

    Ok(Json(AbsUserDto::from_user(&user, progress_dtos)))
}

pub fn get_me_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(get_me))
}
