use std::cmp::PartialEq;
use chrono::NaiveDateTime;
use crate::constants::inner_constants::{DEFAULT_IMAGE_URL, ENVIRONMENT_SERVICE};
use crate::domain::models::podcast::episode::{PodcastEpisode, PodcastEpisodeStatus};

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PodcastEpisodeDto {
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
    pub(crate) status: String,
    pub(crate) download_time: Option<NaiveDateTime>,
    pub(crate) guid: String,
    pub(crate) deleted: bool,
    pub (crate) episode_numbering_processed : bool,
}


impl From<PodcastEpisode> for PodcastEpisodeDto {
    fn from(value: PodcastEpisode) -> Self {
        PodcastEpisodeDto {
            id: value.id,
            podcast_id: value.podcast_id,
            episode_id: value.episode_id,
            name: value.name,
            url: value.url.clone(),
            date_of_recording: value.date_of_recording,
            image_url: value.image_url.clone(),
            total_time: value.total_time,
            local_url: Self::map_url(&value.local_url, &value.status, &value.url),
            local_image_url: Self::map_url(&value.local_image_url, &value.status, &value.image_url),
            description: value.description,
            status: value.status.to_string(),
            download_time: value.download_time,
            guid: value.guid,
            deleted: value.deleted,
            episode_numbering_processed: value.episode_numbering_processed,
        }
    }
}

impl PodcastEpisodeDto {

    fn map_url(image_url: &str, status: &PodcastEpisodeStatus, remote_url: &str) -> String {
        match image_url == DEFAULT_IMAGE_URL {
            true => {
                let env = ENVIRONMENT_SERVICE.get().unwrap();

                env.server_url.clone().to_owned() + DEFAULT_IMAGE_URL
            }
            false => {
                if status.clone() == PodcastEpisodeStatus::Downloaded {
                    let env = ENVIRONMENT_SERVICE.get().unwrap();

                    env.server_url.clone().to_owned() + image_url
                } else {
                    remote_url.to_owned()
                }
            }
        }
    }
}