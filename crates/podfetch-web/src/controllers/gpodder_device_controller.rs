//! Web-UI endpoints that let a user decide which of their GPodder devices
//! (e.g. AntennaPod) share subscriptions with the web interface.
//!
//! The GPodder API exposes the same information via `/api/2/sync-devices`,
//! but most clients offer no way to configure it, so the web UI does.

use crate::app_state::AppState;
use crate::services::device_sync_group::service::DeviceSyncGroupService;
use axum::extract::{Path, State};
use axum::{Extension, Json};
use common_infrastructure::error::CustomError;
use common_infrastructure::error::CustomErrorInner;
use common_infrastructure::error::ErrorSeverity::Warning;
use common_infrastructure::runtime::DEFAULT_DEVICE;
use podfetch_domain::device::kind::is_castable;
use podfetch_domain::user::User;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GPodderDeviceResponse {
    /// Device id the client registered with, e.g. `antennapod_pixel8`.
    pub device_id: String,
    /// Human readable caption sent by the client.
    pub name: String,
    /// `mobile`, `desktop`, `laptop`, `server` or `other`.
    pub kind: String,
    /// Whether subscriptions are shared with the web interface.
    pub synced_with_web: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GPodderDeviceWebSyncRequest {
    pub enabled: bool,
}

fn list_devices(state: &AppState, user: &User) -> Result<Vec<GPodderDeviceResponse>, CustomError> {
    let groups = state.device_sync_group_service.get_by_user_id(user.id)?;
    let mut devices: Vec<GPodderDeviceResponse> = state
        .device_service
        .query_by_user_id(user.id)?
        .into_iter()
        .filter(|d| !is_castable(&d.kind) && d.deviceid != DEFAULT_DEVICE)
        .map(|d| GPodderDeviceResponse {
            synced_with_web: DeviceSyncGroupService::is_synced_with(
                &groups,
                &d.deviceid,
                DEFAULT_DEVICE,
            ),
            name: if d.name.is_empty() {
                d.deviceid.clone()
            } else {
                d.name
            },
            device_id: d.deviceid,
            kind: d.kind,
        })
        .collect();
    devices.sort_by_key(|d| d.name.to_lowercase());
    Ok(devices)
}

#[utoipa::path(
    get,
    path = "/gpodder/devices",
    responses(
        (status = 200, description = "GPodder devices of the current user", body = Vec<GPodderDeviceResponse>)
    ),
    tag = "gpodder"
)]
pub async fn get_gpodder_devices(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<Vec<GPodderDeviceResponse>>, CustomError> {
    list_devices(&state, &user).map(Json)
}

#[utoipa::path(
    put,
    path = "/gpodder/devices/{device_id}/web-sync",
    params(("device_id" = String, Path, description = "GPodder device id")),
    request_body = GPodderDeviceWebSyncRequest,
    responses(
        (status = 200, description = "Updated GPodder devices of the current user", body = Vec<GPodderDeviceResponse>),
        (status = 404, description = "Device not found")
    ),
    tag = "gpodder"
)]
pub async fn update_gpodder_device_web_sync(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path(device_id): Path<String>,
    Json(request): Json<GPodderDeviceWebSyncRequest>,
) -> Result<Json<Vec<GPodderDeviceResponse>>, CustomError> {
    if !list_devices(&state, &user)?
        .iter()
        .any(|d| d.device_id == device_id)
    {
        return Err(CustomErrorInner::NotFound(Warning).into());
    }

    state.device_sync_group_service.set_synced_with(
        user.id,
        &device_id,
        DEFAULT_DEVICE,
        request.enabled,
    )?;
    list_devices(&state, &user).map(Json)
}

pub fn get_gpodder_device_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_gpodder_devices))
        .routes(routes!(update_gpodder_device_web_sync))
}

#[cfg(test)]
mod tests {
    use super::GPodderDeviceResponse;
    use crate::app_state::AppState;
    use crate::gpodder_api::auth::test_support::tests::create_auth_gpodder;
    use crate::subscription::SubscriptionChangesToClient;
    use crate::test_support::tests::handle_test_startup;
    use crate::test_utils::test_builder::device_test_builder::tests::DevicePostTestDataBuilder;
    use crate::test_utils::test_builder::user_test_builder::tests::UserTestDataBuilder;
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn test_toggle_web_sync_for_gpodder_device() {
        let mut server = handle_test_startup().await;
        let user = AppState::new()
            .user_admin_service
            .create_user(UserTestDataBuilder::new().build())
            .unwrap();
        create_auth_gpodder(&mut server, &user).await;
        let username = user.username.clone();

        let device = DevicePostTestDataBuilder::new().build();
        let response = server
            .test_server
            .post(&format!(
                "/api/2/devices/{username}/{}.json",
                device.caption
            ))
            .json(&device)
            .await;
        assert_eq!(response.status_code(), 200);

        let feed = "https://example.com/web-sync-feed.xml";
        let response = server
            .test_server
            .post(&format!("/api/2/subscriptions/{username}/webview.json"))
            .json(&serde_json::json!({ "add": [feed], "remove": [] }))
            .await;
        assert_eq!(response.status_code(), 200);

        let devices = server
            .test_server
            .get("/api/v1/gpodder/devices")
            .await
            .json::<Vec<GPodderDeviceResponse>>();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].device_id, device.caption);
        assert!(!devices[0].synced_with_web);

        let response = server
            .test_server
            .put(&format!(
                "/api/v1/gpodder/devices/{}/web-sync",
                device.caption
            ))
            .json(&serde_json::json!({ "enabled": true }))
            .await;
        assert_eq!(response.status_code(), 200);
        assert!(response.json::<Vec<GPodderDeviceResponse>>()[0].synced_with_web);

        let changes = server
            .test_server
            .get(&format!(
                "/api/2/subscriptions/{username}/{}.json?since=0",
                device.caption
            ))
            .await
            .json::<SubscriptionChangesToClient>();
        assert_eq!(changes.add, vec![feed.to_string()]);

        let response = server
            .test_server
            .put(&format!(
                "/api/v1/gpodder/devices/{}/web-sync",
                device.caption
            ))
            .json(&serde_json::json!({ "enabled": false }))
            .await;
        assert_eq!(response.status_code(), 200);
        assert!(!response.json::<Vec<GPodderDeviceResponse>>()[0].synced_with_web);

        let response = server
            .test_server
            .put("/api/v1/gpodder/devices/unknown-device/web-sync")
            .json(&serde_json::json!({ "enabled": true }))
            .await;
        assert_eq!(response.status_code(), 404);
    }
}
