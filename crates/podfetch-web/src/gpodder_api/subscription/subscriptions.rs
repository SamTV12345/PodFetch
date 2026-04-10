use crate::app_state::AppState;
use crate::gpodder::{ensure_session_user, map_gpodder_error};
use crate::subscription::{
    SubscriptionChangesToClient, SubscriptionPostResponse, SubscriptionRetrieveRequest,
    SubscriptionUpdateRequest, build_opml, to_client_changes,
};
use axum::extract::{Path, Query, State};
use axum::http::Response;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use common_infrastructure::error::CustomError;
use common_infrastructure::error::CustomErrorInner;
use common_infrastructure::error::ErrorSeverity::Warning;
use common_infrastructure::path::trim_from_path;
use common_infrastructure::time::get_current_timestamp;
use podfetch_domain::session::Session;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[utoipa::path(
    get,
    path="/subscriptions/{username}/{deviceid}",
    request_body=SubscriptionRetrieveRequest,
    responses(
        (status = 200, description = "Gets all subscriptions for a device"),
        (status = 403, description = "Forbidden")
    ),
    tag="gpodder"
)]
pub async fn get_subscriptions(
    State(state): State<AppState>,
    Path(paths): Path<(String, String)>,
    Extension(flag): Extension<Session>,
    Query(query): Query<SubscriptionRetrieveRequest>,
) -> Result<Json<SubscriptionChangesToClient>, CustomError> {
    let username = paths.clone().0;
    let deviceid = trim_from_path(&paths.1);
    ensure_session_user::<CustomError>(&flag.username, &username).map_err(map_gpodder_error)?;

    state
        .subscription_service
        .get_device_subscriptions(deviceid.0, &username, query.since)
        .map(Json)
}

#[utoipa::path(
    get,
    path="/subscriptions/{username}",
    request_body=SubscriptionRetrieveRequest,
    responses(
        (status = 200, description = "Gets all subscriptions"),
        (status = 403, description = "Forbidden")
    ),
    tag="gpodder"
)]
pub async fn get_subscriptions_all(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Extension(flag): Extension<Session>,
    Query(query): Query<SubscriptionRetrieveRequest>,
) -> Result<impl IntoResponse, CustomError> {
    let username = trim_from_path(&username);
    ensure_session_user::<CustomError>(&flag.username, username.0).map_err(map_gpodder_error)?;

    let changes = state
        .subscription_service
        .get_user_subscriptions(&flag.username, query.since)?;

    if username.1 == "opml" {
        Ok(Response::builder()
            .header("Content-Type", "text/x-opml+xml")
            .body(build_opml(&changes.add).to_string().unwrap())
            .unwrap()
            .into_response())
    } else {
        Ok(Json(to_client_changes(changes)).into_response())
    }
}

#[utoipa::path(
    post,
    path="/subscriptions/{username}/{deviceid}",
    request_body=SubscriptionUpdateRequest,
    responses(
        (status = 200, description = "Uploads subscription changes"),
        (status = 403, description = "Forbidden")
    ),
    tag="gpodder"
)]
pub async fn upload_subscription_changes(
    State(state): State<AppState>,
    Extension(flag): Extension<Session>,
    paths: Path<(String, String)>,
    upload_request: Json<SubscriptionUpdateRequest>,
) -> Result<Json<SubscriptionPostResponse>, CustomError> {
    let username = paths.clone().0;
    let deviceid = trim_from_path(&paths.1);
    ensure_session_user::<CustomError>(&flag.username, &username).map_err(map_gpodder_error)?;

    let update_urls =
        state
            .subscription_service
            .update_subscriptions(deviceid.0, &username, upload_request.0)?;

    Ok(Json(SubscriptionPostResponse {
        update_urls,
        timestamp: get_current_timestamp(),
    }))
}

/// Parses podcast URLs from a request body based on format.
fn parse_urls_from_body(body: &str, format: &str) -> Result<Vec<String>, CustomError> {
    match format {
        "json" => serde_json::from_str::<Vec<String>>(body).map_err(|e| {
            CustomErrorInner::BadRequest(format!("Invalid JSON: {e}"), Warning).into()
        }),
        "opml" => {
            let opml = opml::OPML::from_str(body).map_err(|e| {
                CustomError::from(CustomErrorInner::BadRequest(
                    format!("Invalid OPML: {e}"),
                    Warning,
                ))
            })?;
            Ok(opml
                .body
                .outlines
                .iter()
                .filter_map(|o| o.xml_url.clone())
                .collect())
        }
        "txt" => Ok(body
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(String::from)
            .collect()),
        _ => Err(CustomErrorInner::BadRequest(
            "Unsupported format. Use .json, .opml, or .txt".to_string(),
            Warning,
        )
        .into()),
    }
}

#[utoipa::path(
    put,
    path="/subscriptions/{username}/{deviceid}",
    responses(
        (status = 200, description = "Replaces all subscriptions for a device"),
        (status = 403, description = "Forbidden")
    ),
    tag="gpodder"
)]
pub async fn put_device_subscriptions(
    State(state): State<AppState>,
    Extension(flag): Extension<Session>,
    Path(paths): Path<(String, String)>,
    body: String,
) -> Result<Json<SubscriptionPostResponse>, CustomError> {
    let username = paths.0.clone();
    let deviceid = trim_from_path(&paths.1);
    ensure_session_user::<CustomError>(&flag.username, &username).map_err(map_gpodder_error)?;

    let new_urls = parse_urls_from_body(&body, deviceid.1)?;
    let current_urls = state
        .subscription_service
        .get_active_device_podcast_urls(deviceid.0, &username)?;

    let to_add: Vec<String> = new_urls
        .iter()
        .filter(|u| !current_urls.contains(u))
        .cloned()
        .collect();
    let to_remove: Vec<String> = current_urls
        .iter()
        .filter(|u| !new_urls.contains(u))
        .cloned()
        .collect();

    let update_urls = state.subscription_service.update_subscriptions(
        deviceid.0,
        &username,
        crate::subscription::SubscriptionUpdateRequest {
            add: to_add,
            remove: to_remove,
        },
    )?;

    Ok(Json(SubscriptionPostResponse {
        update_urls,
        timestamp: get_current_timestamp(),
    }))
}

pub fn get_subscription_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(upload_subscription_changes))
        .routes(routes!(get_subscriptions_all))
        .routes(routes!(get_subscriptions))
        .route(
            "/subscriptions/{username}/{deviceid}",
            axum::routing::put(put_device_subscriptions),
        )
}

#[utoipa::path(
    get,
    path="/{username}",
    responses(
        (status = 200, description = "Gets all subscriptions (Simple API)"),
        (status = 403, description = "Forbidden"),
        (status = 400, description = "Unsupported format")
    ),
    tag="gpodder"
)]
pub async fn get_simple_subscriptions(
    State(state): State<AppState>,
    Path(username_with_ext): Path<String>,
    Extension(flag): Extension<Session>,
) -> Result<impl IntoResponse, CustomError> {
    let (username, format) = trim_from_path(&username_with_ext);
    ensure_session_user::<CustomError>(&flag.username, username).map_err(map_gpodder_error)?;

    let changes = state
        .subscription_service
        .get_user_subscriptions(&flag.username, 0)?;

    match format {
        "opml" => {
            let xml = build_opml(&changes.add).to_string().unwrap();
            Ok(Response::builder()
                .header("Content-Type", "text/x-opml+xml")
                .body(xml)
                .unwrap()
                .into_response())
        }
        "json" => {
            let urls: Vec<String> = changes.add.iter().map(|s| s.podcast.clone()).collect();
            Ok(Json(urls).into_response())
        }
        "txt" => {
            let text = changes
                .add
                .iter()
                .map(|s| s.podcast.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            Ok(Response::builder()
                .header("Content-Type", "text/plain")
                .body(text)
                .unwrap()
                .into_response())
        }
        _ => Err(CustomErrorInner::BadRequest(
            "Unsupported format. Use .opml, .json, or .txt".to_string(),
            Warning,
        )
        .into()),
    }
}

#[utoipa::path(
    put,
    path="/{username}/{deviceid}",
    responses(
        (status = 200, description = "Replaces all subscriptions for a device (Simple API)"),
        (status = 403, description = "Forbidden")
    ),
    tag="gpodder"
)]
pub async fn put_simple_subscriptions(
    State(state): State<AppState>,
    Extension(flag): Extension<Session>,
    Path(paths): Path<(String, String)>,
    body: String,
) -> Result<Json<SubscriptionPostResponse>, CustomError> {
    let username = &paths.0;
    let deviceid = trim_from_path(&paths.1);
    ensure_session_user::<CustomError>(&flag.username, username).map_err(map_gpodder_error)?;

    let new_urls = parse_urls_from_body(&body, deviceid.1)?;
    let current_urls = state
        .subscription_service
        .get_active_device_podcast_urls(deviceid.0, username)?;

    let to_add: Vec<String> = new_urls
        .iter()
        .filter(|u| !current_urls.contains(u))
        .cloned()
        .collect();
    let to_remove: Vec<String> = current_urls
        .iter()
        .filter(|u| !new_urls.contains(u))
        .cloned()
        .collect();

    let update_urls = state.subscription_service.update_subscriptions(
        deviceid.0,
        username,
        crate::subscription::SubscriptionUpdateRequest {
            add: to_add,
            remove: to_remove,
        },
    )?;

    Ok(Json(SubscriptionPostResponse {
        update_urls,
        timestamp: get_current_timestamp(),
    }))
}

pub fn get_simple_subscription_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_simple_subscriptions))
        .route(
            "/{username}/{deviceid}",
            axum::routing::put(put_simple_subscriptions),
        )
}

#[cfg(test)]
mod tests {
    use crate::app_state::AppState;
    use crate::gpodder_api::auth::test_support::tests::create_auth_gpodder;
    use crate::subscription::SubscriptionChangesToClient;
    use crate::test_support::tests::{TestServerWrapper, handle_test_startup};
    use crate::test_utils::test_builder::device_test_builder::tests::DevicePostTestDataBuilder;
    use crate::test_utils::test_builder::user_test_builder::tests::UserTestDataBuilder;
    use base64::Engine;
    use base64::engine::general_purpose;
    use serial_test::serial;

    fn app_state() -> AppState {
        AppState::new()
    }

    /// Sets up Basic Auth headers directly on the test server (no login/cookie).
    /// This mirrors how Kodi and other Simple API clients authenticate.
    fn setup_basic_auth(server: &mut TestServerWrapper<'_>, user: &podfetch_domain::user::User) {
        let encoded_auth =
            general_purpose::STANDARD.encode(format!("{}:{}", user.username, "password"));
        server.test_server.clear_headers();
        server
            .test_server
            .add_header("Authorization", format!("Basic {encoded_auth}"));
    }

    #[tokio::test]
    #[serial]
    async fn test_get_subscriptions_without_since() {
        let mut server = handle_test_startup().await;
        let state = app_state();
        let user = state
            .user_admin_service
            .create_user(UserTestDataBuilder::new().build())
            .unwrap();
        create_auth_gpodder(&mut server, &user).await;

        // GET without ?since= should default to 0 and return 200
        let response = server
            .test_server
            .get(&format!("/api/2/subscriptions/{}.json", user.username))
            .await;
        assert_eq!(response.status_code(), 200);
        let json = response.json::<SubscriptionChangesToClient>();
        assert!(json.add.is_empty());
        assert!(json.remove.is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn test_get_subscriptions_opml_content_type() {
        let mut server = handle_test_startup().await;
        let state = app_state();
        let user = state
            .user_admin_service
            .create_user(UserTestDataBuilder::new().build())
            .unwrap();
        create_auth_gpodder(&mut server, &user).await;

        let response = server
            .test_server
            .get(&format!("/api/2/subscriptions/{}.opml", user.username))
            .await;
        assert_eq!(response.status_code(), 200);
        assert_eq!(response.header("Content-Type"), "text/x-opml+xml",);
        let body = response.text();
        assert!(body.contains("<opml"));
    }

    #[tokio::test]
    #[serial]
    async fn test_simple_api_opml() {
        let mut server = handle_test_startup().await;
        let state = app_state();
        let user = state
            .user_admin_service
            .create_user(UserTestDataBuilder::new().build())
            .unwrap();
        setup_basic_auth(&mut server, &user);

        let response = server
            .test_server
            .get(&format!("/subscriptions/{}.opml", user.username))
            .await;
        assert_eq!(response.status_code(), 200);
        assert_eq!(response.header("Content-Type"), "text/x-opml+xml",);
        let body = response.text();
        assert!(body.contains("<opml"));
    }

    #[tokio::test]
    #[serial]
    async fn test_simple_api_json() {
        let mut server = handle_test_startup().await;
        let state = app_state();
        let user = state
            .user_admin_service
            .create_user(UserTestDataBuilder::new().build())
            .unwrap();
        setup_basic_auth(&mut server, &user);

        let response = server
            .test_server
            .get(&format!("/subscriptions/{}.json", user.username))
            .await;
        assert_eq!(response.status_code(), 200);
        let urls = response.json::<Vec<String>>();
        assert!(urls.is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn test_simple_api_txt() {
        let mut server = handle_test_startup().await;
        let state = app_state();
        let user = state
            .user_admin_service
            .create_user(UserTestDataBuilder::new().build())
            .unwrap();
        setup_basic_auth(&mut server, &user);

        let response = server
            .test_server
            .get(&format!("/subscriptions/{}.txt", user.username))
            .await;
        assert_eq!(response.status_code(), 200);
        assert_eq!(response.header("Content-Type"), "text/plain",);
    }

    #[tokio::test]
    #[serial]
    async fn test_simple_api_with_subscriptions() {
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

        // Add a subscription via the Advanced API
        let resp = server
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
        assert_eq!(resp.status_code(), 200);

        // Switch to Basic Auth for Simple API calls
        setup_basic_auth(&mut server, &user);

        // Retrieve via Simple API OPML
        let response = server
            .test_server
            .get(&format!("/subscriptions/{}.opml", user.username))
            .await;
        assert_eq!(response.status_code(), 200);
        let body = response.text();
        assert!(
            body.contains("https://example.com/feed.xml"),
            "OPML should contain the subscribed feed URL"
        );

        // Retrieve via Simple API JSON
        let response = server
            .test_server
            .get(&format!("/subscriptions/{}.json", user.username))
            .await;
        assert_eq!(response.status_code(), 200);
        let urls = response.json::<Vec<String>>();
        assert_eq!(urls, vec!["https://example.com/feed.xml"]);

        // Retrieve via Simple API TXT
        let response = server
            .test_server
            .get(&format!("/subscriptions/{}.txt", user.username))
            .await;
        assert_eq!(response.status_code(), 200);
        let body = response.text();
        assert_eq!(body.trim(), "https://example.com/feed.xml");
    }

    #[tokio::test]
    #[serial]
    async fn test_put_device_subscriptions_json() {
        let mut server = handle_test_startup().await;
        let state = app_state();
        let user = state
            .user_admin_service
            .create_user(UserTestDataBuilder::new().build())
            .unwrap();
        create_auth_gpodder(&mut server, &user).await;

        // Create a device
        let device_post = DevicePostTestDataBuilder::new().build();
        server
            .test_server
            .post(&format!(
                "/api/2/devices/{}/{}",
                user.username, device_post.caption
            ))
            .json(&device_post)
            .await;

        // Add initial subscriptions via POST
        server
            .test_server
            .post(&format!(
                "/api/2/subscriptions/{}/{}.json",
                user.username, device_post.caption
            ))
            .json(&serde_json::json!({
                "add": ["https://example.com/feed1.xml", "https://example.com/feed2.xml"],
                "remove": []
            }))
            .await;

        // PUT replaces all subscriptions — feed1 should be removed, feed3 added
        let response = server
            .test_server
            .put(&format!(
                "/api/2/subscriptions/{}/{}.json",
                user.username, device_post.caption
            ))
            .text(
                &serde_json::json!([
                    "https://example.com/feed2.xml",
                    "https://example.com/feed3.xml"
                ])
                .to_string(),
            )
            .await;
        assert_eq!(response.status_code(), 200);

        // Verify via GET
        let response = server
            .test_server
            .get(&format!(
                "/api/2/subscriptions/{}/{}.json?since=0",
                user.username, device_post.caption
            ))
            .await;
        assert_eq!(response.status_code(), 200);
        let changes = response.json::<SubscriptionChangesToClient>();
        assert!(
            changes
                .add
                .contains(&"https://example.com/feed2.xml".to_string()),
            "feed2 should still be active"
        );
        assert!(
            changes
                .add
                .contains(&"https://example.com/feed3.xml".to_string()),
            "feed3 should be added"
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_put_simple_subscriptions_txt() {
        let mut server = handle_test_startup().await;
        let state = app_state();
        let user = state
            .user_admin_service
            .create_user(UserTestDataBuilder::new().build())
            .unwrap();
        create_auth_gpodder(&mut server, &user).await;

        // Create a device
        let device_post = DevicePostTestDataBuilder::new().build();
        server
            .test_server
            .post(&format!(
                "/api/2/devices/{}/{}",
                user.username, device_post.caption
            ))
            .json(&device_post)
            .await;

        // PUT via Simple API with plaintext
        setup_basic_auth(&mut server, &user);
        let response = server
            .test_server
            .put(&format!(
                "/subscriptions/{}/{}.txt",
                user.username, device_post.caption
            ))
            .text("https://example.com/feed-a.xml\nhttps://example.com/feed-b.xml")
            .await;
        assert_eq!(response.status_code(), 200);

        // Verify via Simple API JSON
        let response = server
            .test_server
            .get(&format!("/subscriptions/{}.json", user.username))
            .await;
        assert_eq!(response.status_code(), 200);
        let urls = response.json::<Vec<String>>();
        assert!(urls.contains(&"https://example.com/feed-a.xml".to_string()));
        assert!(urls.contains(&"https://example.com/feed-b.xml".to_string()));
    }
}
