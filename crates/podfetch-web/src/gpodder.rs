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

pub fn build_client_parametrization(server_url: &str) -> ClientParametrization {
    ClientParametrization {
        mygpo_feedservice: BaseURL {
            base_url: server_url.to_string(),
        },
        mygpo: BaseURL {
            base_url: format!("{server_url}rss"),
        },
        update_timeout: 604800,
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct EpisodeActionPostResponse {
    pub update_urls: Vec<String>,
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

pub fn parse_since_epoch<Err: Display>(
    since: i64,
) -> Result<Option<chrono::NaiveDateTime>, GpodderControllerError<Err>> {
    chrono::DateTime::from_timestamp(since, 0)
        .map(|v| v.naive_utc())
        .ok_or_else(|| GpodderControllerError::BadRequest("invalid 'since' timestamp".to_string()))
        .map(Some)
}
