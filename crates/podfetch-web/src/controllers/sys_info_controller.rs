use crate::app_state::AppState;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::{Extension, Json};
use common_infrastructure::config::ConfigModel;
use common_infrastructure::error::ErrorSeverity::Info;
use common_infrastructure::error::ErrorType::CustomErrorType;
use common_infrastructure::error::{
    ApiError, CustomError, CustomErrorInner, ErrorSeverity, ErrorType,
};
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
fn get_size(path: &str) -> Result<u64, std::io::Error> {
    let mut total: u64 = 0;
    let path = std::path::Path::new(path);
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let meta = entry.metadata()?;
            if meta.is_dir() {
                total += get_size(entry.path().to_str().unwrap_or(""))?;
            } else {
                total += meta.len();
            }
        }
    }
    Ok(total)
}
use crate::sys::{self, LoginControllerError, LoginRequest, SysExtraInfo, VersionInfo};
use crate::url_rewriting::resolve_server_url_from_headers;
use podfetch_domain::user::User;
use reqwest::StatusCode;
use sysinfo::{Disks, System};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[utoipa::path(
get,
path="/sys/info",
responses(
(status = 200, description = "Gets the system information",
body = SysExtraInfo)),
tag="sys"
)]
pub async fn get_sys_info(
    Extension(requester): Extension<User>,
) -> Result<Json<SysExtraInfo>, CustomError> {
    if !requester.is_admin() {
        return Err(CustomErrorInner::Forbidden(Info).into());
    }

    let mut sys = System::new();
    let disks = Disks::new_with_refreshed_list();
    let simplified_disks = disks.iter().map(sys::map_disk).collect();

    sys.refresh_all();
    sys.refresh_cpu_all();

    let podcast_byte_size =
        get_size(&ENVIRONMENT_SERVICE.default_podfetch_folder).map_err(|e| {
            common_infrastructure::error::map_io_error(
                e,
                Some(ENVIRONMENT_SERVICE.default_podfetch_folder.to_string()),
                ErrorSeverity::Critical,
            )
        })?;

    Ok(Json(SysExtraInfo {
        system: sys::map_system(&sys),
        disks: simplified_disks,
        podcast_directory: podcast_byte_size,
    }))
}

#[utoipa::path(
get,
path="/sys/config",
responses(
(status = 200, description = "Gets the environment configuration",
body=ConfigModel)),
tag="sys"
)]
pub async fn get_public_config(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<ConfigModel> {
    let resolved_server_url = resolve_server_url_from_headers(&headers);
    let config = state.environment.get_config(&resolved_server_url);

    Json(sys::get_public_config(config, &resolved_server_url, None))
}

#[utoipa::path(
post,
path="/login",
operation_id="sys_login",
request_body=LoginRequest,
responses(
(status = 200, description = "Performs a login if basic auth is enabled",
body=String)),
tag="sys"
)]
pub async fn login(
    State(state): State<AppState>,
    auth: Json<LoginRequest>,
) -> Result<StatusCode, ErrorType> {
    sys::login(state.login_service.as_ref(), &auth.0)
        .map(|_| StatusCode::OK)
        .map_err(map_login_error)
}

fn map_login_error(error: LoginControllerError<CustomError>) -> ErrorType {
    match error {
        LoginControllerError::Unauthorized => ApiError::wrong_user_or_password().into(),
        LoginControllerError::Forbidden => CustomErrorInner::Forbidden(Info).into(),
        LoginControllerError::Service(error) => CustomErrorType(error),
    }
}

#[utoipa::path(
get,
path="/info",
responses(
(status = 200, description = "Gets the info of the server", body=VersionInfo)),
tag="info"
)]
pub async fn get_info() -> Json<VersionInfo> {
    Json(sys::get_version_info(
        resolve_version(),
        option_env!("GIT_BRANCH").unwrap_or("unknown"),
        resolve_commit(),
        built_info::CI_PLATFORM.unwrap_or("No CI platform"),
        built_info::BUILT_TIME_UTC,
        built_info::CFG_OS,
    ))
}

fn resolve_version() -> &'static str {
    match option_env!("VW_VERSION") {
        Some(v) if v != "unknown" => v,
        _ => env!("CARGO_PKG_VERSION"),
    }
}

fn resolve_commit() -> &'static str {
    match option_env!("GIT_REV") {
        Some(rev) if rev != "unknown" => rev,
        _ => "unknown",
    }
}

pub fn get_sys_info_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_sys_info))
        .routes(routes!(get_info))
}

#[cfg(test)]
mod tests {
    use crate::app_state::AppState;
    use crate::role::Role;
    use crate::test_support::tests::handle_test_startup;
    use axum::Extension;
    use axum::extract::State;
    use chrono::Utc;
    use common_infrastructure::config::ConfigModel;
    use common_infrastructure::error::{CustomErrorInner, ErrorType};
    use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
    use podfetch_domain::user::User;
    use serde_json::Value;
    use serial_test::serial;
    use sha256::digest;
    use uuid::Uuid;

    fn unique_username(prefix: &str) -> String {
        format!("{prefix}-{}", Uuid::new_v4())
    }

    fn insert_user_with_password(username: String, plain_password: &str) {
        let user = User::new(
            Uuid::new_v4(),
            username,
            Role::User,
            Some(digest(plain_password)),
            Utc::now().naive_utc(),
            true,
        );
        app_state().user_admin_service.create_user(user).unwrap();
    }

    fn app_state() -> AppState {
        AppState::new()
    }

    fn assert_client_error_status(status: u16) {
        assert!(
            (400..500).contains(&status),
            "expected 4xx status, got {status}"
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_login_endpoint_returns_200_for_valid_db_user() {
        let server = handle_test_startup().await;
        let username = unique_username("login-ok-user");
        let password = "login-secret-123";
        insert_user_with_password(username.clone(), password);

        let response = server
            .test_server
            .post("/api/v1/login")
            .json(&serde_json::json!({
                "username": username,
                "password": password
            }))
            .await;

        assert_eq!(response.status_code(), 200);
    }

    #[tokio::test]
    #[serial]
    async fn test_login_endpoint_returns_401_for_unknown_user() {
        let server = handle_test_startup().await;

        let response = server
            .test_server
            .post("/api/v1/login")
            .json(&serde_json::json!({
                "username": unique_username("unknown-login-user"),
                "password": "irrelevant"
            }))
            .await;

        assert_eq!(response.status_code(), 401);
        let payload = response.json::<Value>();
        assert_eq!(payload["errorCode"], "WRONG_USER_OR_PASSWORD");
    }

    #[tokio::test]
    #[serial]
    async fn test_login_endpoint_returns_403_for_wrong_password() {
        let server = handle_test_startup().await;
        let username = unique_username("login-wrong-password-user");
        insert_user_with_password(username.clone(), "correct-password");

        let response = server
            .test_server
            .post("/api/v1/login")
            .json(&serde_json::json!({
                "username": username,
                "password": "wrong-password"
            }))
            .await;

        assert_eq!(response.status_code(), 403);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_public_config_uses_forwarded_headers() {
        let mut server = handle_test_startup().await;
        server
            .test_server
            .add_header("x-forwarded-host", "podfetch.example.com");
        server.test_server.add_header("x-forwarded-proto", "https");
        server.test_server.add_header("x-forwarded-prefix", "/ui");

        let response = server.test_server.get("/api/v1/sys/config").await;
        assert_eq!(response.status_code(), 200);

        let config = response.json::<ConfigModel>();
        assert_eq!(config.server_url, "https://podfetch.example.com/ui/");
        assert_eq!(config.rss_feed, "https://podfetch.example.com/ui/rss");
        assert_eq!(config.ws_url, "wss://podfetch.example.com/ui/socket.io");
        assert_eq!(
            config.transcription_enabled,
            ENVIRONMENT_SERVICE.transcription_config.is_some()
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_get_info_endpoint_returns_version_payload() {
        let server = handle_test_startup().await;

        let response = server.test_server.get("/api/v1/info").await;
        assert_eq!(response.status_code(), 200);

        let payload = response.json::<Value>();

        let version = payload["version"]
            .as_str()
            .expect("version must be present");
        let commit = payload["commit"].as_str().expect("commit must be present");
        let os = payload["os"].as_str().expect("os must be present");
        let branch = payload["ref"]
            .as_str()
            .expect("ref (branch) must be present");

        // Never "unknown" in a dev/CI environment with git available
        assert_ne!(
            version, "unknown",
            "version should not be 'unknown' when git is available"
        );
        assert_ne!(
            commit, "unknown",
            "commit should not be 'unknown' when git is available"
        );

        // commit must be exactly 8 hex characters
        assert_eq!(
            commit.len(),
            8,
            "commit '{commit}' must be 8 characters (short git hash)"
        );
        assert!(
            commit.chars().all(|c| c.is_ascii_hexdigit()),
            "commit '{commit}' must be hexadecimal"
        );

        // version format: <tag>-<hash> or <tag>-<hash> (branch)
        // The commit hash must appear in the version string
        assert!(
            version.contains(commit),
            "version '{version}' must contain commit hash '{commit}'"
        );

        // os must match the compile-time target
        assert_eq!(
            os,
            std::env::consts::OS,
            "os must match compile-time target"
        );

        // branch must not be empty
        assert!(!branch.is_empty(), "branch must not be empty");
        assert_ne!(
            branch, "unknown",
            "branch should not be 'unknown' when git is available"
        );
    }

    #[test]
    fn test_get_version_info_maps_fields_correctly() {
        let info = crate::sys::get_version_info(
            "v5.2.2-abc12345",
            "feature/fix-commit-tag",
            "abc12345",
            "GitHub Actions",
            "2025-01-15",
            "linux",
        );

        assert_eq!(info.version, "v5.2.2-abc12345");
        assert_eq!(info.r#ref, "feature/fix-commit-tag");
        assert_eq!(
            info.commit, "abc12345",
            "commit field must contain the hash, not the tag"
        );
        assert_eq!(info.ci, "GitHub Actions");
        assert_eq!(info.time, "2025-01-15");
        assert_eq!(info.os, "linux");
    }

    #[test]
    fn test_resolve_commit_is_not_the_tag() {
        // Ensures the commit field gets GIT_REV (hash), not GIT_EXACT_TAG (tag name).
        // GIT_EXACT_TAG would look like "v5.2.2" or be "unknown",
        // whereas GIT_REV is always 8 hex chars when git is available.
        let commit = super::resolve_commit();
        if commit != "unknown" {
            // Must be 8 hex chars — a tag name like "v5.2.2" would fail here
            assert_eq!(commit.len(), 8, "commit '{commit}' must be 8 chars");
            assert!(
                commit.chars().all(|c| c.is_ascii_hexdigit()),
                "commit '{commit}' must be hex, not a tag name"
            );
        }
    }

    #[test]
    fn test_resolve_version_contains_commit_when_git_available() {
        let version = super::resolve_version();
        let commit = super::resolve_commit();
        if commit != "unknown" {
            assert!(
                version.contains(commit),
                "version '{version}' must contain commit hash '{commit}'"
            );
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_get_sys_info_endpoint_returns_system_payload_for_admin() {
        let server = handle_test_startup().await;

        let response = server.test_server.get("/api/v1/sys/info").await;
        assert_eq!(response.status_code(), 200);

        let payload = response.json::<Value>();
        assert!(payload["podcast_directory"].is_number());
        assert!(payload["system"].is_object());
        assert!(payload["disks"].is_array());
    }

    #[tokio::test]
    #[serial]
    async fn test_login_endpoint_rejects_invalid_payload() {
        let server = handle_test_startup().await;

        let missing_password = server
            .test_server
            .post("/api/v1/login")
            .json(&serde_json::json!({
                "username": unique_username("invalid-login-user")
            }))
            .await;
        assert_client_error_status(missing_password.status_code().as_u16());

        let non_object_payload = server
            .test_server
            .post("/api/v1/login")
            .json(&serde_json::json!(["user", "pass"]))
            .await;
        assert_client_error_status(non_object_payload.status_code().as_u16());
    }

    #[tokio::test]
    #[serial]
    async fn test_get_sys_info_handler_returns_forbidden_for_non_admin() {
        let non_admin = User::new(
            Uuid::new_v4(),
            unique_username("non-admin"),
            Role::User,
            Some(digest("non-admin-password")),
            Utc::now().naive_utc(),
            true,
        );

        let result = super::get_sys_info(Extension(non_admin)).await;
        match result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden error"),
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_sys_endpoints_return_client_error_for_wrong_http_methods() {
        let server = handle_test_startup().await;

        let info_with_put = server.test_server.put("/api/v1/info").await;
        assert_client_error_status(info_with_put.status_code().as_u16());

        let sys_config_with_post = server.test_server.post("/api/v1/sys/config").await;
        assert_client_error_status(sys_config_with_post.status_code().as_u16());

        let sys_info_with_post = server.test_server.post("/api/v1/sys/info").await;
        assert_client_error_status(sys_info_with_post.status_code().as_u16());

        let login_with_get = server.test_server.get("/api/v1/login").await;
        assert_client_error_status(login_with_get.status_code().as_u16());
    }

    #[tokio::test]
    #[serial]
    async fn test_sys_endpoints_return_not_found_for_invalid_paths() {
        let server = handle_test_startup().await;

        let wrong_info_path = server.test_server.get("/api/v1/inf").await;
        assert_eq!(wrong_info_path.status_code(), 404);

        let wrong_sys_path = server.test_server.get("/api/v1/sys/infos").await;
        assert_eq!(wrong_sys_path.status_code(), 404);

        let trailing_slash_path = server.test_server.get("/api/v1/sys/info/").await;
        assert_client_error_status(trailing_slash_path.status_code().as_u16());
    }

    #[tokio::test]
    #[serial]
    async fn test_get_public_config_without_forwarded_headers_returns_defaults() {
        let server = handle_test_startup().await;

        let response = server.test_server.get("/api/v1/sys/config").await;
        assert_eq!(response.status_code(), 200);

        let config = response.json::<ConfigModel>();
        assert!(!config.server_url.is_empty());
        assert!(config.rss_feed.ends_with("/rss") || config.rss_feed.ends_with("rss"));
        assert!(config.ws_url.starts_with("ws://") || config.ws_url.starts_with("wss://"));
    }

    #[tokio::test]
    #[serial]
    async fn test_login_handler_returns_forbidden_for_wrong_password() {
        let username = unique_username("login-handler-user");
        insert_user_with_password(username.clone(), "correct-password");

        let result = super::login(
            State(app_state()),
            axum::Json(super::LoginRequest {
                username,
                password: "wrong-password".to_string(),
            }),
        )
        .await;

        match result {
            Err(ErrorType::CustomErrorType(err)) => {
                assert!(matches!(err.inner, CustomErrorInner::Forbidden(_)))
            }
            _ => panic!("expected forbidden login result"),
        }
    }
}
