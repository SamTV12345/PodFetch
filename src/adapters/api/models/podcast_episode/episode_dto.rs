use std::fmt;
use std::fmt::Display;
use chrono::NaiveDateTime;
use crate::domain::models::episode::episode::Episode;

#[derive(Debug, Deserialize, Serialize, Clone)]
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


impl From<Episode> for EpisodeDto {
    fn from(value: Episode) -> Self {
        EpisodeDto {
            podcast: value.podcast,
            episode: value.episode,
            timestamp: value.timestamp,
            guid: value.guid,
            action: EpisodeAction::from(value.action),
            started: value.started,
            position: value.position,
            total: value.total,
            device: value.device,
        }
    }
}


impl Into<Episode> for EpisodeDto {
    fn into(self) -> Episode {
        Episode {
            id: 0,
            podcast: self.podcast,
            episode: self.episode,
            timestamp: self.timestamp,
            guid: self.guid,
            action: self.action.into(),
            started: self.started,
            position: self.position,
            total: self.total,
            device: self.device,
            username: "".to_string(),
        }
    }
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
#[derive(PartialEq, Clone)]
pub enum EpisodeAction {
    New,
    Download,
    Play,
    Delete,
}

impl Into<String> for EpisodeAction {
    fn into(self) -> String {
        self.to_string()
    }
}

impl From<String> for EpisodeAction {
    fn from(s: String) -> Self {
        EpisodeAction::from_string(&s)
    }
}

impl EpisodeAction {
    pub fn from_string(s: &str) -> Self {
        match s {
            "new" => EpisodeAction::New,
            "download" => EpisodeAction::Download,
            "play" => EpisodeAction::Play,
            "delete" => EpisodeAction::Delete,
            _ => panic!("Unknown episode action: {}", s),
        }
    }
}

impl Display for EpisodeAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EpisodeAction::New => write!(f, "new"),
            EpisodeAction::Download => write!(f, "download"),
            EpisodeAction::Play => write!(f, "play"),
            EpisodeAction::Delete => write!(f, "delete"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum EpisodeActionRaw {
    New,
    Download,
    Play,
    Delete,
}