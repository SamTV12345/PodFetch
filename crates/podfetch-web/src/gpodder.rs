use crate::history::EpisodeDto;
use crate::url_rewriting::normalize_server_url;
use common_infrastructure::error::ErrorSeverity::Warning;
use common_infrastructure::error::{CustomError, CustomErrorInner};
use http::{HeaderMap, header::COOKIE};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize)]
pub struct ClientParametrization {
    pub mygpo: BaseURL,
    #[serde(rename = "mygpo-feedservice")]
    pub mygpo_feedservice: BaseURL,
    pub update_timeout: i32,
}

#[derive(Serialize, Deserialize)]
pub struct BaseURL {
    #[serde(rename = "baseurl")]
    pub base_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct EpisodeActionPostResponse {
    pub update_urls: Vec<String>,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct EpisodeActionResponse {
    pub actions: Vec<EpisodeDto>,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct EpisodeSinceRequest {
    pub since: i64,
    pub podcast: Option<String>,
    pub device: Option<String>,
    pub aggregate: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum GpodderControllerError<E: Display> {
    #[error("forbidden")]
    Forbidden,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("{0}")]
    Service(E),
}

pub fn map_gpodder_error(error: GpodderControllerError<CustomError>) -> CustomError {
    match error {
        GpodderControllerError::Forbidden => CustomErrorInner::Forbidden(Warning).into(),
        GpodderControllerError::BadRequest(message) => {
            CustomErrorInner::BadRequest(message, Warning).into()
        }
        GpodderControllerError::Service(error) => error,
    }
}

pub fn ensure_session_user<Err: Display>(
    session_username: &str,
    requested_username: &str,
) -> Result<(), GpodderControllerError<Err>> {
    if session_username == requested_username {
        Ok(())
    } else {
        Err(GpodderControllerError::Forbidden)
    }
}

pub fn extract_session_cookie_value(headers: &HeaderMap) -> Option<String> {
    headers
        .get(COOKIE)
        .and_then(|header| header.to_str().ok())
        .and_then(|raw_cookie| {
            raw_cookie.split(';').map(str::trim).find_map(|cookie| {
                let (name, value) = cookie.split_once('=')?;
                if name.trim() == "sessionid" {
                    Some(value.trim().to_string())
                } else {
                    None
                }
            })
        })
}

pub fn require_session_cookie<Err: Display>(
    session_cookie: Option<String>,
) -> Result<String, GpodderControllerError<Err>> {
    session_cookie.ok_or(GpodderControllerError::Forbidden)
}

pub fn require_active_session<T, Err: Display>(
    session: Option<T>,
) -> Result<T, GpodderControllerError<Err>> {
    session.ok_or(GpodderControllerError::Forbidden)
}

pub fn require_present_header_value<Err: Display>(
    header_value: Option<&str>,
) -> Result<String, GpodderControllerError<Err>> {
    header_value
        .map(ToString::to_string)
        .ok_or(GpodderControllerError::Forbidden)
}

pub fn require_password_match<Err: Display>(
    password_hash: Option<&str>,
    expected_hash: &str,
) -> Result<(), GpodderControllerError<Err>> {
    match password_hash {
        Some(hash) if hash == expected_hash => Ok(()),
        _ => Err(GpodderControllerError::Forbidden),
    }
}

pub fn parse_since_epoch<Err: Display>(
    since: i64,
) -> Result<Option<chrono::NaiveDateTime>, GpodderControllerError<Err>> {
    chrono::DateTime::from_timestamp(since, 0)
        .map(|v| v.naive_utc())
        .ok_or_else(|| GpodderControllerError::BadRequest("invalid 'since' timestamp".to_string()))
        .map(Some)
}

pub fn build_client_parametrization(server_url: &str) -> ClientParametrization {
    let normalized = normalize_server_url(server_url);
    ClientParametrization {
        mygpo_feedservice: BaseURL {
            base_url: normalized.clone(),
        },
        mygpo: BaseURL {
            base_url: format!("{normalized}rss"),
        },
        update_timeout: 604800,
    }
}
