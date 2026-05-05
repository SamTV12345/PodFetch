use crate::app_state::AppState;
use crate::cast::{
    CastControlRequest, CastDeviceResponse, CastSessionResponse, CastStartRequest,
    CastStatusResponse, DiscoveredCastDeviceResponse, parse_device_uuid, parse_session_id,
};
use crate::usecases::podcast_episode::PodcastEpisodeUseCase;
use axum::extract::{Path, State};
use axum::{Extension, Json};
use common_infrastructure::error::CustomError;
use podfetch_cast::CastMedia;
use podfetch_domain::user::User;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[utoipa::path(
    get,
    path = "/cast/devices",
    responses(
        (status = 200, description = "Chromecast devices visible to the caller", body = Vec<CastDeviceResponse>)
    ),
    tag = "cast"
)]
pub async fn list_cast_devices(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<Vec<CastDeviceResponse>>, CustomError> {
    let devices = state
        .cast_orchestrator
        .list_castable(&user)
        .map_err(CustomError::from)?;
    Ok(Json(
        devices
            .iter()
            .filter_map(CastDeviceResponse::from_device)
            .collect(),
    ))
}

#[utoipa::path(
    post,
    path = "/cast/devices/discover",
    responses(
        (status = 200, description = "Newly discovered Chromecasts on the server's LAN", body = Vec<DiscoveredCastDeviceResponse>),
        (status = 403, description = "Only admins can trigger discovery")
    ),
    tag = "cast"
)]
pub async fn discover_cast_devices(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<Vec<DiscoveredCastDeviceResponse>>, CustomError> {
    let found = state
        .cast_orchestrator
        .discover(&user)
        .await
        .map_err(CustomError::from)?;
    Ok(Json(found.into_iter().map(Into::into).collect::<Vec<_>>()))
}

#[utoipa::path(
    post,
    path = "/cast/sessions",
    request_body = CastStartRequest,
    responses(
        (status = 200, description = "Started a new cast session", body = CastSessionResponse),
        (status = 404, description = "Device not found or not visible to caller")
    ),
    tag = "cast"
)]
pub async fn start_cast_session(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(req): Json<CastStartRequest>,
) -> Result<Json<CastSessionResponse>, CustomError> {
    // Resolve the GUID-like string id used by the watchtime store. If
    // the episode isn't found we still allow the cast to start — the
    // session simply won't persist watchtime.
    let episode_string_id =
        PodcastEpisodeUseCase::get_podcast_episode_by_internal_id(req.episode_id)?
            .map(|e| e.episode_id);

    let media = CastMedia {
        url: req.url,
        mime: req.mime,
        title: req.title,
        artwork_url: req.artwork_url,
        duration_secs: req.duration_secs,
        episode_id: Some(req.episode_id),
    };
    let session = state
        .cast_orchestrator
        .start(&user, &req.chromecast_uuid, media, episode_string_id)
        .await
        .map_err(CustomError::from)?;
    Ok(Json(CastSessionResponse::from_active(&session)))
}

#[utoipa::path(
    post,
    path = "/cast/sessions/{session_id}/control",
    request_body = CastControlRequest,
    responses(
        (status = 200, description = "Command accepted"),
        (status = 403, description = "Caller does not own this session"),
        (status = 404, description = "Session not found")
    ),
    tag = "cast"
)]
pub async fn control_cast_session(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(session_id): Path<String>,
    Json(req): Json<CastControlRequest>,
) -> Result<(), CustomError> {
    let id = parse_session_id(&session_id);
    state
        .cast_orchestrator
        .control(&user, &id, req.into())
        .await
        .map_err(CustomError::from)?;
    Ok(())
}

#[utoipa::path(
    get,
    path = "/cast/sessions/{session_id}",
    responses(
        (status = 200, description = "Snapshot of session status", body = CastStatusResponse),
        (status = 403, description = "Caller does not own this session"),
        (status = 404, description = "Session not found")
    ),
    tag = "cast"
)]
pub async fn get_cast_session_status(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(session_id): Path<String>,
) -> Result<Json<CastStatusResponse>, CustomError> {
    let id = parse_session_id(&session_id);
    let status = state
        .cast_orchestrator
        .status(&user, &id)
        .map_err(CustomError::from)?;
    Ok(Json(status.into()))
}

pub fn get_cast_router() -> OpenApiRouter<AppState> {
    // The unused parser is exported through `parse_device_uuid` for callers
    // that want to construct typed UUIDs from path strings; keeping the
    // import alive avoids dead-code warnings if it's later removed.
    let _ = parse_device_uuid;
    OpenApiRouter::new()
        .routes(routes!(list_cast_devices))
        .routes(routes!(discover_cast_devices))
        .routes(routes!(start_cast_session))
        .routes(routes!(control_cast_session))
        .routes(routes!(get_cast_session_status))
}
