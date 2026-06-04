//! Management API for Mopidy servers (gated behind MOPIDY_INTEGRATION_ENABLED
//! at the router-mount level). A server is stored as a `mopidy_*` Device with
//! a generated `chromecast_uuid` public handle and a `base_url`.

use crate::app_state::AppState;
use crate::services::mopidy::driver::MopidyDriver;
use axum::extract::{Path, State};
use axum::{Extension, Json};
use common_infrastructure::error::{CustomError, CustomErrorInner, ErrorSeverity};
use podfetch_domain::device::{Device, kind as device_kind};
use podfetch_domain::user::User;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

#[derive(Deserialize, Debug, Clone, ToSchema)]
pub struct AddMopidyServerRequest {
    pub name: String,
    pub url: String,
    /// When true the server is visible to every user; otherwise owner-only.
    pub shared: bool,
}

#[derive(Deserialize, Debug, Clone, ToSchema)]
pub struct TestMopidyServerRequest {
    pub url: String,
}

#[derive(Serialize, Debug, Clone, ToSchema)]
pub struct MopidyServerResponse {
    pub id: String,
    pub name: String,
    pub url: String,
    pub kind: String,
}

impl MopidyServerResponse {
    fn from_device(device: &Device) -> Option<Self> {
        Some(Self {
            id: device.id?.to_string(),
            name: device.name.clone(),
            url: device.base_url.clone()?,
            kind: device.kind.clone(),
        })
    }
}

#[derive(Serialize, Debug, Clone, ToSchema)]
pub struct MopidyTestResult {
    pub reachable: bool,
    pub version: Option<String>,
    pub error: Option<String>,
}

fn normalize_url(raw: &str) -> Result<String, CustomError> {
    let trimmed = raw.trim().trim_end_matches('/');
    if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
        return Err(CustomErrorInner::BadRequest(
            "Mopidy URL must start with http:// or https://".to_string(),
            ErrorSeverity::Warning,
        )
        .into());
    }
    Ok(trimmed.to_string())
}

fn require_admin(user: &User) -> Result<(), CustomError> {
    if user.is_admin() {
        Ok(())
    } else {
        Err(CustomErrorInner::Forbidden(ErrorSeverity::Warning).into())
    }
}

#[utoipa::path(
    get,
    path = "/mopidy/servers",
    responses((status = 200, description = "Mopidy servers visible to the caller", body = Vec<MopidyServerResponse>)),
    tag = "mopidy"
)]
pub async fn list_mopidy_servers(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<Vec<MopidyServerResponse>>, CustomError> {
    let devices = state.device_service.list_castable_for_user(user.id)?;
    Ok(Json(
        devices
            .iter()
            .filter(|d| device_kind::is_mopidy(&d.kind))
            .filter_map(MopidyServerResponse::from_device)
            .collect(),
    ))
}

#[utoipa::path(
    post,
    path = "/mopidy/servers",
    request_body = AddMopidyServerRequest,
    responses(
        (status = 200, description = "Server added", body = MopidyServerResponse),
        (status = 400, description = "Invalid URL or server unreachable"),
        (status = 403, description = "Admin only")
    ),
    tag = "mopidy"
)]
pub async fn add_mopidy_server(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(req): Json<AddMopidyServerRequest>,
) -> Result<Json<MopidyServerResponse>, CustomError> {
    require_admin(&user)?;
    let url = normalize_url(&req.url)?;
    MopidyDriver::ping(&url).await.map_err(|e| {
        CustomError::from(CustomErrorInner::BadRequest(
            format!("Mopidy server unreachable: {e}"),
            ErrorSeverity::Warning,
        ))
    })?;

    let kind = if req.shared {
        device_kind::MOPIDY_SHARED
    } else {
        device_kind::MOPIDY_PERSONAL
    };
    let device = Device {
        id: None,
        deviceid: url.clone(),
        kind: kind.to_string(),
        name: req.name,
        user_id: user.id,
        chromecast_uuid: Some(Uuid::new_v4().to_string()),
        agent_id: None,
        last_seen_at: None,
        ip: None,
        base_url: Some(url),
    };
    let created = state.device_service.create(device)?;
    MopidyServerResponse::from_device(&created).map(Json).ok_or_else(|| {
        CustomErrorInner::BadRequest(
            "could not build server response".to_string(),
            ErrorSeverity::Error,
        )
        .into()
    })
}

#[utoipa::path(
    post,
    path = "/mopidy/servers/test",
    request_body = TestMopidyServerRequest,
    responses((status = 200, description = "Connection test result", body = MopidyTestResult)),
    tag = "mopidy"
)]
pub async fn test_mopidy_server(
    Extension(user): Extension<User>,
    Json(req): Json<TestMopidyServerRequest>,
) -> Result<Json<MopidyTestResult>, CustomError> {
    require_admin(&user)?;
    let url = normalize_url(&req.url)?;
    match MopidyDriver::ping(&url).await {
        Ok(version) => Ok(Json(MopidyTestResult {
            reachable: true,
            version: Some(version),
            error: None,
        })),
        Err(e) => Ok(Json(MopidyTestResult {
            reachable: false,
            version: None,
            error: Some(e.to_string()),
        })),
    }
}

#[utoipa::path(
    delete,
    path = "/mopidy/servers/{id}",
    responses(
        (status = 200, description = "Server deleted"),
        (status = 403, description = "Not allowed"),
        (status = 404, description = "Not found")
    ),
    tag = "mopidy"
)]
pub async fn delete_mopidy_server(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(id): Path<String>,
) -> Result<(), CustomError> {
    let uuid = Uuid::parse_str(&id).map_err(|_| {
        CustomError::from(CustomErrorInner::BadRequest(
            "invalid server id".to_string(),
            ErrorSeverity::Warning,
        ))
    })?;
    let device = state
        .device_service
        .find_by_id(uuid)?
        .filter(|d| device_kind::is_mopidy(&d.kind))
        .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(ErrorSeverity::Warning)))?;

    // Admin can delete any; a non-admin may only delete their own personal server.
    let owns = device.user_id == user.id && device.kind == device_kind::MOPIDY_PERSONAL;
    if !user.is_admin() && !owns {
        return Err(CustomErrorInner::Forbidden(ErrorSeverity::Warning).into());
    }
    state.device_service.delete_by_id(uuid)?;
    Ok(())
}

pub fn get_mopidy_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list_mopidy_servers))
        .routes(routes!(add_mopidy_server))
        .routes(routes!(test_mopidy_server))
        .routes(routes!(delete_mopidy_server))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_rejects_non_http_and_trims_slash() {
        assert!(normalize_url("ftp://x").is_err());
        assert_eq!(normalize_url("http://m.local:6680/").unwrap(), "http://m.local:6680");
    }
}
