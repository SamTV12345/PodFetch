use crate::app_state::AppState;
use crate::gpodder::{ensure_session_user, map_gpodder_error};
use axum::extract::{Path, Query, State};
use axum::{Extension, Json};
use common_infrastructure::error::CustomError;
use common_infrastructure::path::trim_from_path;
use podfetch_domain::gpodder_setting::GpodderSetting;
use podfetch_domain::session::Session;
use serde::Deserialize;
use serde_json::Value;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[derive(Deserialize)]
pub struct SettingsQuery {
    pub device: Option<String>,
    pub podcast: Option<String>,
    pub episode: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct SettingsUpdateRequest {
    pub set: Option<serde_json::Map<String, Value>>,
    pub remove: Option<Vec<String>>,
}

fn resolve_scope_id(scope: &str, query: &SettingsQuery) -> Option<String> {
    match scope {
        "account" => None,
        "device" => query.device.clone(),
        "podcast" => query.podcast.clone(),
        "episode" => query.episode.clone(),
        _ => None,
    }
}

#[utoipa::path(
    get,
    path="/settings/{username}/{scope}",
    responses(
        (status = 200, description = "Gets settings for the given scope."),
        (status = 403, description = "Forbidden"),
    ),
    tag="gpodder"
)]
pub async fn get_settings(
    State(state): State<AppState>,
    Path((username, scope)): Path<(String, String)>,
    Query(query): Query<SettingsQuery>,
    Extension(flag): Extension<Session>,
) -> Result<Json<Value>, CustomError> {
    let username = trim_from_path(&username);
    let scope = trim_from_path(&scope);
    ensure_session_user::<CustomError>(&flag.username, username.0).map_err(map_gpodder_error)?;

    let scope_id = resolve_scope_id(scope.0, &query);
    let result =
        state
            .gpodder_setting_service
            .get_setting(flag.user_id, scope.0, scope_id.as_deref())?;

    match result {
        Some(setting) => {
            let data: Value = serde_json::from_str(&setting.data)
                .unwrap_or_else(|_| Value::Object(Default::default()));
            Ok(Json(data))
        }
        None => Ok(Json(Value::Object(Default::default()))),
    }
}

#[utoipa::path(
    post,
    path="/settings/{username}/{scope}",
    responses(
        (status = 200, description = "Updates settings for the given scope."),
        (status = 403, description = "Forbidden"),
    ),
    tag="gpodder"
)]
pub async fn save_settings(
    State(state): State<AppState>,
    Path((username, scope)): Path<(String, String)>,
    Query(query): Query<SettingsQuery>,
    Extension(flag): Extension<Session>,
    Json(body): Json<SettingsUpdateRequest>,
) -> Result<Json<Value>, CustomError> {
    let username = trim_from_path(&username);
    let scope = trim_from_path(&scope);
    ensure_session_user::<CustomError>(&flag.username, username.0).map_err(map_gpodder_error)?;

    let scope_id = resolve_scope_id(scope.0, &query);
    let existing =
        state
            .gpodder_setting_service
            .get_setting(flag.user_id, scope.0, scope_id.as_deref())?;

    let mut data: serde_json::Map<String, Value> = existing
        .as_ref()
        .and_then(|s| serde_json::from_str(&s.data).ok())
        .unwrap_or_default();

    // Merge: add all keys from "set"
    if let Some(set_map) = body.set {
        for (key, value) in set_map {
            data.insert(key, value);
        }
    }

    // Remove all keys in "remove"
    if let Some(remove_keys) = body.remove {
        for key in remove_keys {
            data.remove(&key);
        }
    }

    let data_string = serde_json::to_string(&data).unwrap_or_else(|_| "{}".to_string());

    let setting = GpodderSetting {
        id: existing.map(|s| s.id).unwrap_or(0),
        user_id: flag.user_id,
        scope: scope.0.to_string(),
        scope_id,
        data: data_string,
    };

    state.gpodder_setting_service.save_setting(setting)?;

    Ok(Json(Value::Object(data)))
}

pub fn get_settings_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_settings))
        .routes(routes!(save_settings))
}

#[cfg(test)]
mod tests {
    use crate::app_state::AppState;
    use crate::gpodder_api::auth::test_support::tests::create_auth_gpodder;
    use crate::test_support::tests::handle_test_startup;
    use crate::test_utils::test_builder::user_test_builder::tests::UserTestDataBuilder;
    use serial_test::serial;

    fn app_state() -> AppState {
        AppState::new()
    }

    #[tokio::test]
    #[serial]
    async fn test_get_settings_empty() {
        let mut server = handle_test_startup().await;
        let state = app_state();
        let user = state
            .user_admin_service
            .create_user(UserTestDataBuilder::new().build())
            .unwrap();
        create_auth_gpodder(&mut server, &user).await;

        let response = server
            .test_server
            .get(&format!("/api/2/settings/{}/account.json", user.username))
            .await;
        assert_eq!(response.status_code(), 200);
        let body = response.json::<serde_json::Value>();
        assert_eq!(body, serde_json::json!({}));
    }

    #[tokio::test]
    #[serial]
    async fn test_save_and_get_settings() {
        let mut server = handle_test_startup().await;
        let state = app_state();
        let user = state
            .user_admin_service
            .create_user(UserTestDataBuilder::new().build())
            .unwrap();
        create_auth_gpodder(&mut server, &user).await;

        // Save some account settings
        let response = server
            .test_server
            .post(&format!("/api/2/settings/{}/account.json", user.username))
            .json(&serde_json::json!({
                "set": {
                    "theme": "dark",
                    "language": "en"
                }
            }))
            .await;
        assert_eq!(response.status_code(), 200);
        let body = response.json::<serde_json::Value>();
        assert_eq!(body["theme"], "dark");
        assert_eq!(body["language"], "en");

        // Retrieve
        let response = server
            .test_server
            .get(&format!("/api/2/settings/{}/account.json", user.username))
            .await;
        assert_eq!(response.status_code(), 200);
        let body = response.json::<serde_json::Value>();
        assert_eq!(body["theme"], "dark");
        assert_eq!(body["language"], "en");
    }

    #[tokio::test]
    #[serial]
    async fn test_settings_remove_keys() {
        let mut server = handle_test_startup().await;
        let state = app_state();
        let user = state
            .user_admin_service
            .create_user(UserTestDataBuilder::new().build())
            .unwrap();
        create_auth_gpodder(&mut server, &user).await;

        // Set initial values
        server
            .test_server
            .post(&format!("/api/2/settings/{}/account.json", user.username))
            .json(&serde_json::json!({
                "set": { "a": 1, "b": 2, "c": 3 }
            }))
            .await;

        // Remove "b", update "a"
        let response = server
            .test_server
            .post(&format!("/api/2/settings/{}/account.json", user.username))
            .json(&serde_json::json!({
                "set": { "a": 99 },
                "remove": ["b"]
            }))
            .await;
        assert_eq!(response.status_code(), 200);
        let body = response.json::<serde_json::Value>();
        assert_eq!(body["a"], 99);
        assert!(body.get("b").is_none());
        assert_eq!(body["c"], 3);
    }

    #[tokio::test]
    #[serial]
    async fn test_settings_device_scope() {
        let mut server = handle_test_startup().await;
        let state = app_state();
        let user = state
            .user_admin_service
            .create_user(UserTestDataBuilder::new().build())
            .unwrap();
        create_auth_gpodder(&mut server, &user).await;

        // Save device-scoped settings
        let response = server
            .test_server
            .post(&format!(
                "/api/2/settings/{}/device.json?device=phone",
                user.username
            ))
            .json(&serde_json::json!({
                "set": { "auto_download": true }
            }))
            .await;
        assert_eq!(response.status_code(), 200);

        // Retrieve for same device
        let response = server
            .test_server
            .get(&format!(
                "/api/2/settings/{}/device.json?device=phone",
                user.username
            ))
            .await;
        assert_eq!(response.status_code(), 200);
        let body = response.json::<serde_json::Value>();
        assert_eq!(body["auto_download"], true);

        // Different device should be empty
        let response = server
            .test_server
            .get(&format!(
                "/api/2/settings/{}/device.json?device=tablet",
                user.username
            ))
            .await;
        assert_eq!(response.status_code(), 200);
        let body = response.json::<serde_json::Value>();
        assert_eq!(body, serde_json::json!({}));
    }
}
