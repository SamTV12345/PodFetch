use crate::app_state::AppState;
use crate::audiobookshelf_api::auth_middleware::AuthenticatedUser;
use crate::audiobookshelf_api::dto::media_progress::MediaProgressDto;
use crate::audiobookshelf_api::dto::user::AbsUserDto;
use axum::Json;
use axum::extract::{Query, State};
use common_infrastructure::error::CustomError;
use serde::Deserialize;
use serde_json::{Value, json};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[derive(Deserialize)]
pub struct ListeningSessionsQuery {
    pub limit: Option<i64>,
}

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

#[utoipa::path(
    get,
    path = "/api/me/listening-sessions",
    responses((status = 200, description = "User's listening session history")),
    tag = "audiobookshelf"
)]
pub async fn list_listening_sessions(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Query(query): Query<ListeningSessionsQuery>,
) -> Result<Json<Value>, CustomError> {
    let limit = query.limit.unwrap_or(20).clamp(1, 500);
    let sessions = state
        .audiobookshelf_listening_session_service
        .list_for_user(user.id, limit)?;
    let total = sessions.len() as i64;
    let entries: Vec<Value> = sessions
        .iter()
        .map(|s| {
            json!({
                "id": s.id,
                "userId": s.user_id.to_string(),
                "libraryId": s.library_id,
                "libraryItemId": s.library_item_id,
                "episodeId": s.episode_id,
                "mediaType": s.media_type,
                "playMethod": s.play_method,
                "duration": s.duration,
                "currentTime": s.current_time,
                "timeListening": s.time_listening,
                "startedAt": s.started_at.and_utc().timestamp_millis(),
                "updatedAt": s.updated_at.and_utc().timestamp_millis(),
                "displayTitle": s.display_title,
                "displayAuthor": s.display_author,
                "coverPath": s.cover_path,
            })
        })
        .collect();
    Ok(Json(json!({
        "total": total,
        "limit": limit,
        "page": 0,
        "numPages": 1,
        "itemsPerPage": limit,
        "sessions": entries,
    })))
}

pub fn get_me_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_me))
        .routes(routes!(list_listening_sessions))
}
