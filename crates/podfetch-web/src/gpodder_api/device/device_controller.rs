use crate::app_state::AppState;
use crate::device::{self, DeviceControllerError, DevicePost, DeviceResponse};
use crate::gpodder::{ensure_session_user, map_gpodder_error, parse_since_epoch};
use crate::history::map_episode_to_dto;
use crate::usecases::watchtime::WatchtimeUseCase as WatchtimeService;
use axum::extract::{Path, Query, State};
use axum::{Extension, Json};
use common_infrastructure::error::ErrorSeverity::Warning;
use common_infrastructure::error::{CustomError, CustomErrorInner};
use common_infrastructure::path::trim_from_path;
use common_infrastructure::time::get_current_timestamp;
use podfetch_domain::device_sync_group::DeviceSyncGroup;
use podfetch_domain::session::Session;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

fn map_controller_error(error: DeviceControllerError<CustomError>) -> CustomError {
    match error {
        DeviceControllerError::Forbidden => CustomErrorInner::Forbidden(Warning).into(),
        DeviceControllerError::Service(error) => error,
    }
}

#[utoipa::path(
    post,
    path="/devices/{username}/{deviceid}",
    request_body=DevicePost,
    responses(
        (status = 200, description = "Creates a new device.", body = DeviceResponse),
        (status = 403, description = "Forbidden.")
    ),
    tag="gpodder"
)]
pub async fn post_device(
    State(state): State<AppState>,
    query: Path<(String, String)>,
    Extension(flag): Extension<Session>,
    Json(device_post): Json<DevicePost>,
) -> Result<Json<DeviceResponse>, CustomError> {
    let username = &query.0.0;
    let deviceid = trim_from_path(&query.0.1);
    device::post_device(
        state.device_service.as_ref(),
        &flag.username,
        username,
        deviceid.0,
        device_post,
    )
    .map(Json)
    .map_err(map_controller_error)
}

#[utoipa::path(
    get,
    path="/devices/{username}",
    responses(
        (status = 200, description = "Gets all devices of a user.", body = [DeviceResponse])
    ),
    tag="gpodder"
)]
pub async fn get_devices_of_user(
    State(state): State<AppState>,
    Path(query): Path<String>,
    Extension(flag): Extension<Session>,
) -> Result<Json<Vec<DeviceResponse>>, CustomError> {
    let query = trim_from_path(&query);
    let user_query = query.0;
    device::get_devices_of_user(state.device_service.as_ref(), &flag.username, user_query)
        .map(Json)
        .map_err(map_controller_error)
}

#[derive(Deserialize)]
pub struct DeviceUpdatesQuery {
    #[serde(default)]
    pub since: i64,
    pub include_actions: Option<bool>,
}

#[derive(Serialize)]
pub struct DeviceUpdatesResponse {
    pub add: Vec<String>,
    pub remove: Vec<String>,
    pub updates: Vec<serde_json::Value>,
    pub timestamp: i64,
}

#[utoipa::path(
    get,
    path="/updates/{username}/{deviceid}",
    responses(
        (status = 200, description = "Gets subscription and episode updates for a device."),
        (status = 403, description = "Forbidden"),
    ),
    tag="gpodder"
)]
pub async fn get_device_updates(
    State(state): State<AppState>,
    Path(paths): Path<(String, String)>,
    Extension(flag): Extension<Session>,
    Query(query): Query<DeviceUpdatesQuery>,
) -> Result<Json<DeviceUpdatesResponse>, CustomError> {
    let username = &paths.0;
    let deviceid = trim_from_path(&paths.1);
    ensure_session_user::<CustomError>(&flag.username, username).map_err(map_gpodder_error)?;

    let sub_changes = state.subscription_service.get_device_subscriptions(
        deviceid.0,
        username,
        query.since as i32,
    )?;

    let since_date = parse_since_epoch::<CustomError>(query.since).map_err(map_gpodder_error)?;
    let actions = WatchtimeService::get_actions_by_username(
        username,
        since_date,
        Some(deviceid.0.to_string()),
        None,
        None,
    )?;

    let include_actions = query.include_actions.unwrap_or(false);
    let updates: Vec<serde_json::Value> = actions
        .iter()
        .map(|episode| {
            let dto = map_episode_to_dto(&episode.clone().into());
            let mut obj = serde_json::json!({
                "podcast_url": dto.podcast,
                "episode": dto.episode,
                "action": dto.action.to_string(),
            });
            if include_actions {
                obj["action_detail"] = serde_json::json!({
                    "started": dto.started,
                    "position": dto.position,
                    "total": dto.total,
                });
            }
            obj
        })
        .collect();

    Ok(Json(DeviceUpdatesResponse {
        add: sub_changes.add,
        remove: sub_changes.remove,
        updates,
        timestamp: get_current_timestamp(),
    }))
}

#[derive(Serialize)]
pub struct SyncDevicesResponse {
    pub synchronized: Vec<Vec<String>>,
    #[serde(rename = "not-synchronized")]
    pub not_synchronized: Vec<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct SyncDevicesRequest {
    pub synchronize: Option<Vec<Vec<String>>>,
    #[serde(rename = "stop-synchronize")]
    pub stop_synchronize: Option<Vec<Vec<String>>>,
}

#[utoipa::path(
    get,
    path="/sync-devices/{username}",
    responses(
        (status = 200, description = "Gets the device sync status."),
        (status = 403, description = "Forbidden"),
    ),
    tag="gpodder"
)]
pub async fn get_sync_devices(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Extension(flag): Extension<Session>,
) -> Result<Json<SyncDevicesResponse>, CustomError> {
    let username = trim_from_path(&username);
    ensure_session_user::<CustomError>(&flag.username, username.0).map_err(map_gpodder_error)?;

    let all_devices = state.device_service.query_by_username(username.0)?;
    let sync_groups = state
        .device_sync_group_service
        .get_by_username(username.0)?;

    // Group by group_id
    let mut groups: std::collections::HashMap<i32, Vec<String>> = std::collections::HashMap::new();
    let mut synced_device_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

    for sg in &sync_groups {
        groups
            .entry(sg.group_id)
            .or_default()
            .push(sg.device_id.clone());
        synced_device_ids.insert(sg.device_id.clone());
    }

    let synchronized: Vec<Vec<String>> = groups.into_values().collect();
    let not_synchronized: Vec<String> = all_devices
        .iter()
        .filter(|d| !synced_device_ids.contains(&d.deviceid))
        .map(|d| d.deviceid.clone())
        .collect();

    Ok(Json(SyncDevicesResponse {
        synchronized,
        not_synchronized,
    }))
}

#[utoipa::path(
    post,
    path="/sync-devices/{username}",
    responses(
        (status = 200, description = "Updates the device sync configuration."),
        (status = 403, description = "Forbidden"),
    ),
    tag="gpodder"
)]
pub async fn post_sync_devices(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Extension(flag): Extension<Session>,
    Json(body): Json<SyncDevicesRequest>,
) -> Result<Json<SyncDevicesResponse>, CustomError> {
    let username_parsed = trim_from_path(&username);
    ensure_session_user::<CustomError>(&flag.username, username_parsed.0)
        .map_err(map_gpodder_error)?;

    let mut sync_groups = state
        .device_sync_group_service
        .get_by_username(username_parsed.0)?;

    let max_group_id = sync_groups.iter().map(|g| g.group_id).max().unwrap_or(0);

    // Add new sync groups
    if let Some(synchronize) = body.synchronize {
        for (i, group) in synchronize.iter().enumerate() {
            let group_id = max_group_id + 1 + i as i32;
            // Remove these devices from any existing group first
            sync_groups.retain(|g| !group.contains(&g.device_id));
            for device_id in group {
                sync_groups.push(DeviceSyncGroup {
                    id: 0,
                    username: username_parsed.0.to_string(),
                    group_id,
                    device_id: device_id.clone(),
                });
            }
        }
    }

    // Remove sync groups
    if let Some(stop_synchronize) = body.stop_synchronize {
        for group in stop_synchronize {
            sync_groups.retain(|g| !group.contains(&g.device_id));
        }
    }

    state
        .device_sync_group_service
        .replace_all(username_parsed.0, sync_groups)?;

    // Return updated state
    get_sync_devices(State(state), Path(username), Extension(flag)).await
}

#[cfg(test)]
mod tests {
    use crate::app_state::AppState;
    use crate::gpodder_api::auth::test_support::tests::create_auth_gpodder;
    use crate::test_support::tests::handle_test_startup;
    use crate::test_utils::test_builder::device_test_builder::tests::DevicePostTestDataBuilder;
    use crate::test_utils::test_builder::user_test_builder::tests::UserTestDataBuilder;
    use serial_test::serial;

    fn app_state() -> AppState {
        AppState::new()
    }

    #[tokio::test]
    #[serial]
    async fn test_device_updates() {
        let mut server = handle_test_startup().await;
        let state = app_state();
        let user = state
            .user_admin_service
            .create_user(UserTestDataBuilder::new().build())
            .unwrap();
        create_auth_gpodder(&mut server, &user).await;

        // Create a device
        let device_post = DevicePostTestDataBuilder::new().build();
        let resp = server
            .test_server
            .post(&format!(
                "/api/2/devices/{}/{}",
                user.username, device_post.caption
            ))
            .json(&device_post)
            .await;
        assert_eq!(resp.status_code(), 200);

        // Add a subscription
        server
            .test_server
            .post(&format!(
                "/api/2/subscriptions/{}/{}.json",
                user.username, device_post.caption
            ))
            .json(&serde_json::json!({
                "add": ["https://example.com/feed.xml"],
                "remove": []
            }))
            .await;

        // Get device updates
        let response = server
            .test_server
            .get(&format!(
                "/api/2/updates/{}/{}.json?since=0",
                user.username, device_post.caption
            ))
            .await;
        assert_eq!(response.status_code(), 200);
        let body = response.json::<serde_json::Value>();
        assert!(
            body["add"]
                .as_array()
                .unwrap()
                .contains(&serde_json::json!("https://example.com/feed.xml"))
        );
        assert!(body["timestamp"].as_i64().is_some());
    }

    #[tokio::test]
    #[serial]
    async fn test_sync_devices_empty() {
        let mut server = handle_test_startup().await;
        let state = app_state();
        let user = state
            .user_admin_service
            .create_user(UserTestDataBuilder::new().build())
            .unwrap();
        create_auth_gpodder(&mut server, &user).await;

        // Create two devices
        let device1 = DevicePostTestDataBuilder::new().build();
        server
            .test_server
            .post(&format!(
                "/api/2/devices/{}/{}",
                user.username, device1.caption
            ))
            .json(&device1)
            .await;

        let response = server
            .test_server
            .get(&format!("/api/2/sync-devices/{}.json", user.username))
            .await;
        assert_eq!(response.status_code(), 200);
        let body = response.json::<serde_json::Value>();
        assert!(body["synchronized"].as_array().unwrap().is_empty());
        assert!(
            body["not-synchronized"]
                .as_array()
                .unwrap()
                .contains(&serde_json::json!(device1.caption))
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_sync_devices_synchronize_and_stop() {
        let mut server = handle_test_startup().await;
        let state = app_state();
        let user = state
            .user_admin_service
            .create_user(UserTestDataBuilder::new().build())
            .unwrap();
        create_auth_gpodder(&mut server, &user).await;

        // Create two devices
        let device1 = DevicePostTestDataBuilder::new().build();
        server
            .test_server
            .post(&format!(
                "/api/2/devices/{}/{}",
                user.username, device1.caption
            ))
            .json(&device1)
            .await;

        let device2 = DevicePostTestDataBuilder::new().build();
        server
            .test_server
            .post(&format!(
                "/api/2/devices/{}/{}",
                user.username, device2.caption
            ))
            .json(&device2)
            .await;

        // Synchronize the two devices
        let response = server
            .test_server
            .post(&format!("/api/2/sync-devices/{}.json", user.username))
            .json(&serde_json::json!({
                "synchronize": [[device1.caption, device2.caption]]
            }))
            .await;
        assert_eq!(response.status_code(), 200);
        let body = response.json::<serde_json::Value>();
        let synced = body["synchronized"].as_array().unwrap();
        assert_eq!(synced.len(), 1);
        let group = synced[0].as_array().unwrap();
        assert!(group.contains(&serde_json::json!(device1.caption)));
        assert!(group.contains(&serde_json::json!(device2.caption)));
        assert!(body["not-synchronized"].as_array().unwrap().is_empty());

        // Stop synchronizing
        let response = server
            .test_server
            .post(&format!("/api/2/sync-devices/{}.json", user.username))
            .json(&serde_json::json!({
                "stop-synchronize": [[device1.caption, device2.caption]]
            }))
            .await;
        assert_eq!(response.status_code(), 200);
        let body = response.json::<serde_json::Value>();
        assert!(body["synchronized"].as_array().unwrap().is_empty());
    }
}

pub fn get_device_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_devices_of_user))
        .routes(routes!(post_device))
        .routes(routes!(get_device_updates))
        .routes(routes!(get_sync_devices))
        .routes(routes!(post_sync_devices))
}
