use common_infrastructure::config::ConfigModel;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use url::Url;
use utoipa::ToSchema;

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

#[derive(Debug, Serialize, ToSchema)]
pub struct SysExtraInfo {
    pub system: SystemDto,
    pub disks: Vec<SimplifiedDisk>,
    pub podcast_directory: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SystemDto {
    pub mem_total: u64,
    pub mem_available: u64,
    pub swap_total: u64,
    pub swap_used: u64,
    pub cpus: CpusWrapperDto,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CpusWrapperDto {
    pub global: f32,
    pub cpus: Vec<Cpu>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Cpu {
    pub name: String,
    pub vendor_id: String,
    pub usage: CpuUsageDto,
    pub brand: String,
    pub frequency: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CpuUsageDto {
    pub percent: f32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SimplifiedDisk {
    pub name: String,
    pub total_space: u64,
    pub available_space: u64,
}

pub enum LoginDecision {
    Authenticated,
    WrongUserOrPassword,
    Forbidden,
}

pub trait LoginApplicationService {
    type Error;

    fn verify_login(
        &self,
        username: &str,
        password: &str,
    ) -> Result<LoginDecision, Self::Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum LoginControllerError<E: Display> {
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("{0}")]
    Service(E),
}

pub fn login<S>(
    service: &S,
    request: &LoginRequest,
) -> Result<(), LoginControllerError<S::Error>>
where
    S: LoginApplicationService,
    S::Error: Display,
{
    match service
        .verify_login(&request.username, &request.password)
        .map_err(LoginControllerError::Service)?
    {
        LoginDecision::Authenticated => Ok(()),
        LoginDecision::WrongUserOrPassword => Err(LoginControllerError::Unauthorized),
        LoginDecision::Forbidden => Err(LoginControllerError::Forbidden),
    }
}

pub fn get_public_config(
    mut config: ConfigModel,
    resolved_server_url: &str,
    rewritten_oidc_redirect_uri: Option<String>,
) -> ConfigModel {
    config.server_url = normalize_server_url(resolved_server_url);
    config.rss_feed = format!("{}rss", config.server_url);
    config.ws_url = build_ws_url_from_server_url(&config.server_url);
    if let Some(oidc_config) = config.oidc_config.as_mut()
        && let Some(redirect_uri) = rewritten_oidc_redirect_uri
    {
        oidc_config.redirect_uri = redirect_uri;
    }
    config
}

pub fn get_version_info(
    version: &'static str,
    git_ref: &'static str,
    commit: &'static str,
    ci: &'static str,
    time: &'static str,
    os: &'static str,
) -> VersionInfo {
    VersionInfo {
        version,
        r#ref: git_ref,
        commit,
        ci,
        time,
        os,
    }
}

fn normalize_server_url(server_url: &str) -> String {
    if server_url.ends_with('/') {
        server_url.to_string()
    } else {
        format!("{server_url}/")
    }
}

fn build_ws_url_from_server_url(server_url: &str) -> String {
    let normalized = normalize_server_url(server_url);
    if let Ok(mut parsed) = Url::parse(&normalized) {
        let ws_scheme = if parsed.scheme() == "https" {
            "wss"
        } else {
            "ws"
        };
        if parsed.set_scheme(ws_scheme).is_ok() {
            let mut path = parsed.path().trim_end_matches('/').to_string();
            path.push('/');
            path.push_str("socket.io");
            parsed.set_path(&path);
            return parsed.to_string();
        }
    }
    normalized
}
