use chrono::NaiveDateTime;
use podfetch_domain::episode::Episode;
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;
use url::Url;
use utoipa::ToSchema;


fn deserialize_flexible_timestamp<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = String::deserialize(deserializer)?;

    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&raw) {
        return Ok(dt.naive_utc());
    }

    NaiveDateTime::parse_from_str(&raw, "%Y-%m-%dT%H:%M:%S%.f")
        .or_else(|_| NaiveDateTime::parse_from_str(&raw, "%Y-%m-%dT%H:%M:%S"))
        .map_err(serde::de::Error::custom)
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
pub struct EpisodeDto {
    pub podcast: String,
    pub episode: String,
    #[serde(deserialize_with = "deserialize_flexible_timestamp")]
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
        id: uuid::Uuid::nil(),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn expected_timestamp() -> NaiveDateTime {
        NaiveDateTime::parse_from_str("2026-06-12T08:08:06", "%Y-%m-%dT%H:%M:%S").unwrap()
    }

    fn dto_json(timestamp: &str) -> String {
        format!(
            r#"[{{
                "podcast": "https://example.com/feed.rss",
                "episode": "https://example.com/ep.mp3",
                "action": "play",
                "timestamp": "{timestamp}",
                "position": 120,
                "started": 0,
                "total": 3600,
                "device": "yourpods-ios"
            }}]"#
        )
    }

    #[test]
    fn deserializes_rfc3339_timestamp_with_utc_z_suffix() {
        let dtos: Vec<EpisodeDto> = serde_json::from_str(&dto_json("2026-06-12T08:08:06Z"))
            .expect("RFC 3339 timestamp with Z suffix should deserialize");
        assert_eq!(dtos[0].timestamp, expected_timestamp());
    }

    #[test]
    fn deserializes_rfc3339_timestamp_with_numeric_offset() {
        let dtos: Vec<EpisodeDto> = serde_json::from_str(&dto_json("2026-06-12T10:08:06+02:00"))
            .expect("RFC 3339 timestamp with numeric offset should deserialize");
        // +02:00 normalises to 08:08:06 UTC
        assert_eq!(dtos[0].timestamp, expected_timestamp());
    }

    #[test]
    fn deserializes_naive_timestamp_without_timezone() {
        let dtos: Vec<EpisodeDto> = serde_json::from_str(&dto_json("2026-06-12T08:08:06"))
            .expect("naive timestamp without timezone should still deserialize");
        assert_eq!(dtos[0].timestamp, expected_timestamp());
    }
}
