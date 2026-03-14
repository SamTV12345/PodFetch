use crate::models::user::User;
use axum::{Extension, Json};

use fs_extra::dir::get_size;
use reqwest::StatusCode;
use sha256::digest;

use sysinfo::{Disk, Disks, System};
pub mod built_info {
    // The file has been placed there by the build script.
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

    let sim_disks = disks
        .iter()
        .map(|disk| disk.into())
        .collect::<Vec<SimplifiedDisk>>();

    sys.refresh_all();
    sys.refresh_cpu_all();

    let podcast_byte_size =
        get_size(&ENVIRONMENT_SERVICE.default_podfetch_folder).map_err(|e| {
            map_io_extra_error(
                e,
                Some(
                    ENVIRONMENT_SERVICE
                        .default_podfetch_folder
                        .to_string()
                        .to_string(),
                ),
                ErrorSeverity::Critical,
            )
        })?;
    Ok(Json(SysExtraInfo {
        system: sys.into(),
        disks: sim_disks,
        podcast_directory: podcast_byte_size,
    }))
}
use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::models::settings::ConfigModel;
use crate::utils::error::ErrorSeverity::Info;
use crate::utils::error::ErrorType::CustomErrorType;
use crate::utils::error::{
    ApiError, CustomError, CustomErrorInner, ErrorSeverity, ErrorType, map_io_extra_error,
};
use crate::utils::url_builder::{
    build_ws_url_from_server_url, resolve_server_url_from_headers, rewrite_env_server_url_prefix,
};
use axum::http::HeaderMap;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[derive(Debug, Serialize, ToSchema)]
pub struct SysExtraInfo {
    pub system: SystemDto,
    pub disks: Vec<SimplifiedDisk>,
    pub podcast_directory: u64,
}

impl From<System> for SystemDto {
    fn from(sys: System) -> Self {
        SystemDto {
            mem_total: sys.total_memory(),
            mem_available: sys.available_memory(),
            swap_total: sys.total_swap(),
            swap_used: sys.used_swap(),
            cpus: CpusWrapperDto {
                global: sys.global_cpu_usage(),
                cpus: sys
                    .cpus()
                    .iter()
                    .map(|cpu| Cpu {
                        name: cpu.name().to_string(),
                        vendor_id: cpu.vendor_id().to_string(),
                        usage: CpuUsageDto {
                            percent: cpu.cpu_usage(),
                        },
                        brand: cpu.brand().to_string(),
                        frequency: cpu.frequency(),
                    })
                    .collect(),
            },
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SystemDto {
    mem_total: u64,
    mem_available: u64,
    swap_total: u64,
    swap_used: u64,
    cpus: CpusWrapperDto,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CpusWrapperDto {
    global: f32,
    cpus: Vec<Cpu>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Cpu {
    name: String,
    vendor_id: String,
    usage: CpuUsageDto,
    brand: String,
    frequency: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CpuUsageDto {
    percent: f32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SimplifiedDisk {
    pub name: String,
    pub total_space: u64,
    pub available_space: u64,
}

impl From<&Disk> for SimplifiedDisk {
    fn from(disk: &Disk) -> Self {
        SimplifiedDisk {
            name: disk.name().to_str().unwrap_or("").to_string(),
            total_space: disk.total_space(),
            available_space: disk.available_space(),
        }
    }
}

#[utoipa::path(
get,
path="/sys/config",
responses(
(status = 200, description = "Gets the environment configuration",
body=ConfigModel)),
tag="sys"
)]

pub async fn get_public_config(headers: HeaderMap) -> Json<ConfigModel> {
    let resolved_server_url = resolve_server_url_from_headers(&headers);
    let mut config = ENVIRONMENT_SERVICE.get_config();
    config.server_url = resolved_server_url.clone();
    config.rss_feed = format!("{}rss", resolved_server_url);
    config.ws_url = build_ws_url_from_server_url(&resolved_server_url);
    if let Some(oidc_config) = config.oidc_config.as_mut() {
        oidc_config.redirect_uri =
            rewrite_env_server_url_prefix(&oidc_config.redirect_uri, &resolved_server_url);
    }
    Json(config)
}

#[utoipa::path(
post,
path="/login",
request_body=LoginRequest,
responses(
(status = 200, description = "Performs a login if basic auth is enabled",
body=String)),
tag="sys"
)]
pub async fn login(auth: Json<LoginRequest>) -> Result<StatusCode, ErrorType> {
    use crate::ENVIRONMENT_SERVICE;

    let digested_password = digest(auth.0.password);
    if let Some(admin_username) = &ENVIRONMENT_SERVICE.username
        && admin_username == &auth.0.username
        && let Some(admin_password) = &ENVIRONMENT_SERVICE.password
        && admin_password == &digested_password
    {
        return Ok(StatusCode::OK);
    }
    let db_user = match User::find_by_username(&auth.0.username) {
        Ok(user) => user,
        Err(err) => {
            if matches!(err.inner, CustomErrorInner::NotFound(_)) {
                log::warn!("Login failed for user {}", auth.0.username);
                return Err(ApiError::wrong_user_or_password().into());
            }
            return Err(CustomErrorType(err));
        }
    };

    if db_user.password.is_none() {
        log::warn!("Login failed for user {}", auth.0.username);
        return Err(CustomErrorInner::Forbidden(Info).into());
    }

    if db_user.password.unwrap() == digested_password {
        return Ok(StatusCode::OK);
    }
    log::warn!("Login failed for user {}", auth.0.username);
    Err(CustomErrorInner::Forbidden(Info).into())
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct VersionInfo {
    pub version: &'static str,
    pub r#ref: &'static str,
    pub commit: &'static str,
    pub ci: &'static str,
    pub time: &'static str,
    pub os: &'static str,
}

#[utoipa::path(
get,
path="/info",
responses(
(status = 200, description = "Gets the info of the server", body=VersionInfo)),
tag="info"
)]
pub async fn get_info() -> Json<VersionInfo> {
    let version = VersionInfo {
        commit: env!("GIT_EXACT_TAG"),
        version: env!("VW_VERSION"),
        r#ref: env!("GIT_BRANCH"),
        ci: built_info::CI_PLATFORM.unwrap_or("No CI platform"),
        time: built_info::BUILT_TIME_UTC,
        os: built_info::CFG_OS,
    };
    Json(version)
}

pub fn get_sys_info_router() -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(get_sys_info))
        .routes(routes!(get_info))
}

#[cfg(test)]
mod tests {
    use crate::commands::startup::tests::handle_test_startup;
    use crate::constants::inner_constants::Role;
    use crate::models::settings::ConfigModel;
    use crate::models::user::User;
    use crate::utils::error::CustomErrorInner;
    use axum::Extension;
    use chrono::Utc;
    use serde_json::Value;
    use serial_test::serial;
    use sha256::digest;
    use uuid::Uuid;

    fn unique_username(prefix: &str) -> String {
        format!("{prefix}-{}", Uuid::new_v4())
    }

    fn insert_user_with_password(username: String, plain_password: &str) {
        let mut user = User::new(
            0,
            username,
            Role::User,
            Some(digest(plain_password)),
            Utc::now().naive_utc(),
            true,
        );
        user.insert_user().unwrap();
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
    }

    #[tokio::test]
    #[serial]
    async fn test_get_info_endpoint_returns_version_payload() {
        let server = handle_test_startup().await;

        let response = server.test_server.get("/api/v1/info").await;
        assert_eq!(response.status_code(), 200);

        let payload = response.json::<Value>();
        assert!(payload["version"].as_str().is_some());
        assert!(payload["commit"].as_str().is_some());
        assert!(payload["os"].as_str().is_some());
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
            0,
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
}

