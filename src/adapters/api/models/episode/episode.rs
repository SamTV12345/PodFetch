use chrono::NaiveDateTime;
use crate::domain::models::episode::episode::Episode;

pub struct EpisodeDto {
    pub id: i32,
    pub username: String,
    pub device: String,
    pub podcast: String,
    pub episode: String,
    pub timestamp: NaiveDateTime,
    pub guid: Option<String>,
    pub action: String,
    pub started: Option<i32>,
    pub position: Option<i32>,
    pub total: Option<i32>,
}

impl From<Episode> for EpisodeDto {
    fn from(value: Episode) -> Self {
        EpisodeDto {
            id: value.id,
            username: value.username,
            device: value.device,
            podcast: value.podcast,
            episode: value.episode,
            timestamp: value.timestamp,
            guid: value.guid,
            action: value.action,
            started: value.started,
            position: value.position,
            total: value.total,
        }
    }
}

impl Into<Episode> for EpisodeDto {
    fn into(self) -> Episode {
        Episode {
            id: self.id,
            username: self.username,
            device: self.device,
            podcast: self.podcast,
            episode: self.episode,
            timestamp: self.timestamp,
            guid: self.guid,
            action: self.action,
            started: self.started,
            position: self.position,
            total: self.total,
        }
    }
}
