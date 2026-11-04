//! Web-UI endpoints for managing GPodder sync groups of the current user.
//!
//! Same data and semantics as GPodder's `/api/2/sync-devices`: every group is
//! a set of devices that share their subscriptions, a device belongs to at
//! most one group. The web interface takes part as the device `webview`;
//! devices grouped with it also keep the web favorites in sync. Most clients
//! (e.g. AntennaPod) cannot configure groups themselves, so the web UI does.

use crate::app_state::AppState;
use crate::services::device_sync_group::service::DeviceSyncGroupService;
use crate::services::subscription::service::SubscriptionService;
use axum::extract::State;
use axum::{Extension, Json};
use common_infrastructure::error::CustomError;
use common_infrastructure::error::CustomErrorInner;
use common_infrastructure::error::ErrorSeverity::Warning;
use common_infrastructure::runtime::DEFAULT_DEVICE;
use podfetch_domain::device::kind::is_castable;
use podfetch_domain::user::User;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

/// `kind` of the pseudo device that represents the web interface.
pub const WEB_DEVICE_KIND: &str = "webview";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GPodderDeviceResponse {
    /// Device id the client registered with, e.g. `antennapod_pixel8`.
    pub device_id: String,
    /// Human readable caption sent by the client.
    pub name: String,
    /// `mobile`, `desktop`, `laptop`, `server`, `other` or `webview` for the
    /// web interface itself.
    pub kind: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GPodderSyncGroupResponse {
    pub group_id: i32,
    pub devices: Vec<GPodderDeviceResponse>,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GPodderSyncGroupsResponse {
    pub groups: Vec<GPodderSyncGroupResponse>,
    pub not_synchronized: Vec<GPodderDeviceResponse>,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GPodderSyncGroupsUpdateRequest {
    /// The complete new configuration: one list of device ids per group.
    /// Groups with fewer than two devices are dropped.
    pub groups: Vec<Vec<String>>,
}

/// All devices that can take part in sync groups: the user's GPodder devices,
/// the web interface and any id that is already part of a group.
fn known_devices(state: &AppState, user: &User) -> Result<Vec<GPodderDeviceResponse>, CustomError> {
    let mut devices: Vec<GPodderDeviceResponse> = state
        .device_service
        .query_by_user_id(user.id)?
        .into_iter()
        .filter(|d| !is_castable(&d.kind) && d.deviceid != DEFAULT_DEVICE)
        .map(|d| GPodderDeviceResponse {
            name: if d.name.is_empty() {
                d.deviceid.clone()
            } else {
                d.name
            },
            device_id: d.deviceid,
            kind: d.kind,
        })
        .collect();
    devices.push(GPodderDeviceResponse {
        device_id: DEFAULT_DEVICE.to_string(),
        name: "PodFetch Web".to_string(),
        kind: WEB_DEVICE_KIND.to_string(),
    });
    for group in state.device_sync_group_service.get_by_user_id(user.id)? {
        if !devices.iter().any(|d| d.device_id == group.device_id) {
            devices.push(GPodderDeviceResponse {
                name: group.device_id.clone(),
                device_id: group.device_id,
                kind: "other".to_string(),
            });
        }
    }
    // Web interface first, then alphabetically.
    devices.sort_by_key(|d| (d.kind != WEB_DEVICE_KIND, d.name.to_lowercase()));
    Ok(devices)
}

fn sync_groups_response(
    state: &AppState,
    user: &User,
) -> Result<GPodderSyncGroupsResponse, CustomError> {
    let devices = known_devices(state, user)?;
    let rows = state.device_sync_group_service.get_by_user_id(user.id)?;

    let mut groups: BTreeMap<i32, Vec<GPodderDeviceResponse>> = BTreeMap::new();
    let mut not_synchronized = vec![];
    for device in devices {
        match DeviceSyncGroupService::group_of(&rows, &device.device_id) {
            Some(group_id) => groups.entry(group_id).or_default().push(device),
            None => not_synchronized.push(device),
        }
    }

    Ok(GPodderSyncGroupsResponse {
        groups: groups
            .into_iter()
            .map(|(group_id, devices)| GPodderSyncGroupResponse { group_id, devices })
            .collect(),
        not_synchronized,
    })
}

#[utoipa::path(
    get,
    path = "/gpodder/sync-groups",
    responses(
        (status = 200, description = "GPodder sync groups of the current user", body = GPodderSyncGroupsResponse)
    ),
    tag = "gpodder"
)]
pub async fn get_gpodder_sync_groups(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<GPodderSyncGroupsResponse>, CustomError> {
    sync_groups_response(&state, &user).map(Json)
}

#[utoipa::path(
    put,
    path = "/gpodder/sync-groups",
    request_body = GPodderSyncGroupsUpdateRequest,
    responses(
        (status = 200, description = "Updated GPodder sync groups of the current user", body = GPodderSyncGroupsResponse),
        (status = 400, description = "Unknown device or device in more than one group")
    ),
    tag = "gpodder"
)]
pub async fn update_gpodder_sync_groups(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Json(request): Json<GPodderSyncGroupsUpdateRequest>,
) -> Result<Json<GPodderSyncGroupsResponse>, CustomError> {
    let known: HashSet<String> = known_devices(&state, &user)?
        .into_iter()
        .map(|d| d.device_id)
        .collect();
    let rows = DeviceSyncGroupService::build_groups(user.id, &request.groups, &known)
        .map_err(|message| CustomError::from(CustomErrorInner::BadRequest(message, Warning)))?;

    let previous = state.device_sync_group_service.get_by_user_id(user.id)?;
    let web_members = |rows: &[podfetch_domain::device_sync_group::DeviceSyncGroup]| {
        DeviceSyncGroupService::group_of(rows, DEFAULT_DEVICE)
            .map(|group_id| {
                rows.iter()
                    .filter(|r| r.group_id == group_id)
                    .map(|r| r.device_id.clone())
                    .collect::<HashSet<_>>()
            })
            .unwrap_or_default()
    };
    let previous_web_members = web_members(&previous);
    let joined_web = web_members(&rows)
        .into_iter()
        .any(|device| device != DEFAULT_DEVICE && !previous_web_members.contains(&device));

    state.device_sync_group_service.replace_all(user.id, rows)?;

    if joined_web {
        // Podcasts the newly joined devices subscribed to become web favorites.
        let urls = state
            .subscription_service
            .get_active_device_podcast_urls(DEFAULT_DEVICE, user.id)?;
        SubscriptionService::sync_favorites(user.id, &urls, &[]);
    }

    sync_groups_response(&state, &user).map(Json)
}

pub fn get_gpodder_device_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_gpodder_sync_groups))
        .routes(routes!(update_gpodder_sync_groups))
}

#[cfg(test)]
mod tests {
    use super::GPodderSyncGroupsResponse;
    use crate::app_state::AppState;
    use crate::gpodder_api::auth::test_support::tests::create_auth_gpodder;
    use crate::subscription::SubscriptionChangesToClient;
    use crate::test_support::tests::{TestServerWrapper, handle_test_startup};
    use crate::test_utils::test_builder::device_test_builder::tests::DevicePostTestDataBuilder;
    use crate::test_utils::test_builder::user_test_builder::tests::UserTestDataBuilder;
    use serial_test::serial;

    async fn register_device(server: &TestServerWrapper<'_>, username: &str) -> String {
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
        device.caption
    }

    async fn put_groups(
        server: &TestServerWrapper<'_>,
        groups: serde_json::Value,
    ) -> (u16, Option<GPodderSyncGroupsResponse>) {
        let response = server
            .test_server
            .put("/api/v1/gpodder/sync-groups")
            .json(&serde_json::json!({ "groups": groups }))
            .await;
        let status = response.status_code().as_u16();
        let body = (status == 200).then(|| response.json::<GPodderSyncGroupsResponse>());
        (status, body)
    }

    async fn device_subscriptions(
        server: &TestServerWrapper<'_>,
        username: &str,
        device: &str,
    ) -> SubscriptionChangesToClient {
        server
            .test_server
            .get(&format!(
                "/api/2/subscriptions/{username}/{device}.json?since=0"
            ))
            .await
            .json::<SubscriptionChangesToClient>()
    }

    fn group_ids(response: &GPodderSyncGroupsResponse) -> Vec<Vec<String>> {
        response
            .groups
            .iter()
            .map(|g| {
                let mut ids: Vec<String> = g.devices.iter().map(|d| d.device_id.clone()).collect();
                ids.sort();
                ids
            })
            .collect()
    }

    #[tokio::test]
    #[serial]
    async fn test_manage_sync_groups() {
        let mut server = handle_test_startup().await;
        let user = AppState::new()
            .user_admin_service
            .create_user(UserTestDataBuilder::new().build())
            .unwrap();
        create_auth_gpodder(&mut server, &user).await;
        let username = user.username.clone();
        let phone = register_device(&server, &username).await;
        let tablet = register_device(&server, &username).await;
        let laptop = register_device(&server, &username).await;

        let feed = "https://example.com/group-management-feed.xml";
        server
            .test_server
            .post(&format!("/api/2/subscriptions/{username}/webview.json"))
            .json(&serde_json::json!({ "add": [feed], "remove": [] }))
            .await;

        // Nothing grouped yet; the web interface is listed as a device.
        let initial = server
            .test_server
            .get("/api/v1/gpodder/sync-groups")
            .await
            .json::<GPodderSyncGroupsResponse>();
        assert!(initial.groups.is_empty());
        assert_eq!(initial.not_synchronized.len(), 4);
        assert_eq!(initial.not_synchronized[0].kind, "webview");

        // Two independent groups.
        let (status, body) = put_groups(
            &server,
            serde_json::json!([[phone, "webview"], [tablet, laptop]]),
        )
        .await;
        assert_eq!(status, 200);
        assert_eq!(body.unwrap().groups.len(), 2);
        assert_eq!(
            device_subscriptions(&server, &username, &phone).await.add,
            vec![feed.to_string()]
        );
        assert!(
            device_subscriptions(&server, &username, &laptop)
                .await
                .add
                .is_empty()
        );

        // The GPodder API sees the same groups.
        let sync = server
            .test_server
            .get(&format!("/api/2/sync-devices/{username}.json"))
            .await
            .json::<serde_json::Value>();
        assert_eq!(sync["synchronized"].as_array().unwrap().len(), 2);

        // Merge both groups.
        let (_, body) = put_groups(
            &server,
            serde_json::json!([[phone, "webview", tablet, laptop]]),
        )
        .await;
        let body = body.unwrap();
        assert_eq!(group_ids(&body).len(), 1);
        assert_eq!(group_ids(&body)[0].len(), 4);
        assert_eq!(
            device_subscriptions(&server, &username, &laptop).await.add,
            vec![feed.to_string()]
        );

        // Invalid configurations are rejected and change nothing.
        let (status, _) = put_groups(&server, serde_json::json!([[phone, "unknown-device"]])).await;
        assert_eq!(status, 400);
        let (status, _) = put_groups(
            &server,
            serde_json::json!([[phone, "webview"], [phone, tablet]]),
        )
        .await;
        assert_eq!(status, 400);

        // Dissolve everything.
        let (_, body) = put_groups(&server, serde_json::json!([])).await;
        let body = body.unwrap();
        assert!(body.groups.is_empty());
        assert_eq!(body.not_synchronized.len(), 4);
    }

    fn is_favorite(user_id: uuid::Uuid, podcast_id: &str) -> bool {
        use podfetch_domain::favorite::FavoriteRepository;
        podfetch_persistence::favorite::DieselFavoriteRepository::new(
            podfetch_persistence::db::database(),
        )
        .find_favored_by_user_id(user_id)
        .unwrap()
        .iter()
        .any(|f| f.favored && f.podcast_id.to_string() == podcast_id)
    }

    #[tokio::test]
    #[serial]
    async fn test_web_group_keeps_favorites_in_sync() {
        let mut server = handle_test_startup().await;
        let user = AppState::new()
            .user_admin_service
            .create_user(UserTestDataBuilder::new().build())
            .unwrap();
        create_auth_gpodder(&mut server, &user).await;
        let username = user.username.clone();

        let feed = "https://example.com/favorite-sync-feed.xml";
        let podcast = crate::services::podcast::service::PodcastService::add_podcast_to_database(
            "Favorite Sync Podcast",
            "favorite-sync",
            feed,
            "http://localhost:8080/ui/default.jpg",
            "favorite-sync",
        )
        .unwrap();

        let device = register_device(&server, &username).await;
        let subscribe = |add: Vec<&str>, remove: Vec<&str>| {
            server
                .test_server
                .post(&format!("/api/2/subscriptions/{username}/{device}.json"))
                .json(&serde_json::json!({ "add": add, "remove": remove }))
        };

        // Subscribed in the app before it was grouped with the web: no favorite.
        assert_eq!(subscribe(vec![feed], vec![]).await.status_code(), 200);
        assert!(!is_favorite(user.id, &podcast.id));

        // Joining the web interface's group takes over existing subscriptions.
        let (status, _) = put_groups(&server, serde_json::json!([[device, "webview"]])).await;
        assert_eq!(status, 200);
        assert!(is_favorite(user.id, &podcast.id));

        // Unsubscribing in the app removes the favorite, subscribing adds it.
        assert_eq!(subscribe(vec![], vec![feed]).await.status_code(), 200);
        assert!(!is_favorite(user.id, &podcast.id));
        assert_eq!(subscribe(vec![feed], vec![]).await.status_code(), 200);
        assert!(is_favorite(user.id, &podcast.id));

        // Removing the favorite in the web UI removes it from the app too.
        let podcast_id = uuid::Uuid::parse_str(&podcast.id).unwrap();
        crate::services::podcast::service::PodcastService::update_favor_podcast(
            podcast_id, false, user.id,
        )
        .unwrap();
        let changes = device_subscriptions(&server, &username, &device).await;
        assert!(!changes.add.contains(&feed.to_string()));
        assert!(changes.remove.contains(&feed.to_string()));

        // Unknown feeds are ignored instead of failing the sync.
        assert_eq!(
            subscribe(vec!["https://example.com/not-in-podfetch.xml"], vec![])
                .await
                .status_code(),
            200
        );
    }
}
