use std::fmt::{Display, Formatter};
use std::str::FromStr;
use chrono::NaiveDateTime;
use crate::utils::error::CustomError;

#[derive(Clone, Debug, Default)]
pub struct PodcastEpisode {
    pub(crate) id: i32,
    pub(crate) podcast_id: i32,
    pub(crate) episode_id: String,
    pub(crate) name: String,
    pub(crate) url: String,
    pub(crate) date_of_recording: String,
    pub image_url: String,
    pub total_time: i32,
    pub(crate) local_url: String,
    pub(crate) local_image_url: String,
    pub(crate) description: String,
    pub(crate) status: PodcastEpisodeStatus,
    pub(crate) download_time: Option<NaiveDateTime>,
    pub(crate) guid: String,
    pub(crate) deleted: bool,
    pub(crate) file_episode_path: Option<String>,
    pub(crate) file_image_path: Option<String>,
    pub (crate) episode_numbering_processed : bool,
}

impl PodcastEpisode {
    pub fn check_if_downloaded(&self) -> bool {
        self.status.is_downloaded()
    }
}

#[derive(Clone, Debug, Default)]
pub enum PodcastEpisodeStatus {
    #[default]
    New,
    Downloaded,
    Pending
}

impl FromStr for PodcastEpisodeStatus {
    type Err = CustomError;

    fn from_str(s: &str) -> Result<Self, Err> {
        match s {
            "N" => Ok(PodcastEpisodeStatus::New),
            "D" => Ok(PodcastEpisodeStatus::Downloaded),
            "P" => Ok(PodcastEpisodeStatus::Pending),
            _ => Err(CustomError::Unknown),
        }
    }
}


impl Display for PodcastEpisodeStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PodcastEpisodeStatus::New => write!(f, "N"),
            PodcastEpisodeStatus::Downloaded => write!(f, "D"),
            PodcastEpisodeStatus::Pending => write!(f, "P"),
        }
    }
}

impl PodcastEpisodeStatus {
    pub fn is_downloaded(&self) -> bool {
        match self {
            PodcastEpisodeStatus::Downloaded => true,
            _ => false,
        }
    }
}
