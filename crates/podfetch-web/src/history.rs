use chrono::NaiveDateTime;
use podfetch_domain::episode::Episode;
use serde::{Deserialize, Serialize};
use std::fmt;
use url::Url;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
pub struct EpisodeDto {
    pub podcast: String,
    pub episode: String,
    pub timestamp: NaiveDateTime,
    pub guid: Option<String>,
    pub action: EpisodeAction,
    pub started: Option<i32>,
    pub position: Option<i32>,
    pub total: Option<i32>,
    pub device: String,
}

#[derive(Serialize, Deserialize, Debug, ToSchema, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum EpisodeAction {
    New,
    Download,
    Play,
    Delete,
}

impl EpisodeAction {
    pub fn from_string(s: &str) -> Self {
        match s {
            "new" => EpisodeAction::New,
            "download" => EpisodeAction::Download,
            "play" => EpisodeAction::Play,
            "delete" => EpisodeAction::Delete,
            _ => panic!("Unknown episode action: {s}"),
        }
    }
}

impl fmt::Display for EpisodeAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EpisodeAction::New => write!(f, "new"),
            EpisodeAction::Download => write!(f, "download"),
            EpisodeAction::Play => write!(f, "play"),
            EpisodeAction::Delete => write!(f, "delete"),
        }
    }
}

pub fn map_episode_to_dto(episode: &Episode) -> EpisodeDto {
    EpisodeDto {
        podcast: episode.podcast.clone(),
        episode: episode.episode.clone(),
        timestamp: episode.timestamp,
        guid: episode.guid.clone(),
        action: EpisodeAction::from_string(&episode.action),
        started: episode.started,
        position: episode.position,
        total: episode.total,
        device: episode.device.clone(),
    }
}

pub fn map_episode_dto_to_episode(episode_dto: &EpisodeDto, username: String) -> Episode {
    let mut episode = Url::parse(&episode_dto.episode).unwrap();
    episode.set_query(None);

    Episode {
        id: 0,
        username,
        device: episode_dto.device.clone(),
        podcast: episode_dto.podcast.clone(),
        episode: episode_dto.episode.clone(),
        timestamp: episode_dto.timestamp,
        guid: episode_dto.guid.clone(),
        action: episode_dto.action.clone().to_string(),
        started: episode_dto.started,
        position: episode_dto.position,
        total: episode_dto.total,
    }
}
