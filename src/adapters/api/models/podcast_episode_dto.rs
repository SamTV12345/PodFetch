use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::models::favorite_podcast_episode::FavoritePodcastEpisode;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::user::User;
use chrono::NaiveDateTime;
use std::borrow::Cow;
use std::path::PathBuf;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
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
    pub(crate) episode_numbering_processed: bool,
    pub favored: Option<bool>,
}

impl From<(PodcastEpisode, Option<User>, Option<FavoritePodcastEpisode>)> for PodcastEpisodeDto {
    fn from(value: (PodcastEpisode, Option<User>, Option<FavoritePodcastEpisode>)) -> Self {
        PodcastEpisodeDto {
            id: value.0.id,
            podcast_id: value.0.podcast_id,
            episode_id: value.0.episode_id,
            name: value.0.name,
            url: value.0.url.clone(),
            date_of_recording: value.0.date_of_recording,
            image_url: value.0.image_url.clone(),
            total_time: value.0.total_time,
            local_url: map_file_url(&value.0.file_episode_path, &value.0.url, &value.1),
            local_image_url: map_file_url(&value.0.file_image_path, &value.0.image_url, &value.1),
            description: value.0.description,
            status: value.0.status,
            download_time: value.0.download_time,
            guid: value.0.guid,
            deleted: value.0.deleted,
            episode_numbering_processed: value.0.episode_numbering_processed,
            favored: value.2.map(|f| f.favorite),
        }
    }
}

impl
    From<(
        PodcastEpisode,
        Option<String>,
        Option<FavoritePodcastEpisode>,
    )> for PodcastEpisodeDto
{
    fn from(
        value: (
            PodcastEpisode,
            Option<String>,
            Option<FavoritePodcastEpisode>,
        ),
    ) -> Self {
        PodcastEpisodeDto {
            id: value.0.id,
            podcast_id: value.0.podcast_id,
            episode_id: value.0.episode_id,
            name: value.0.name,
            url: value.0.url.clone(),
            date_of_recording: value.0.date_of_recording,
            image_url: value.0.image_url.clone(),
            total_time: value.0.total_time,
            local_url: map_file_url_with_api_key(
                &value.0.file_episode_path,
                &value.0.url,
                &value.1,
            ),
            local_image_url: map_file_url_with_api_key(
                &value.0.file_image_path,
                &value.0.image_url,
                &value.1,
            ),
            description: value.0.description,
            status: value.0.status,
            download_time: value.0.download_time,
            guid: value.0.guid,
            deleted: value.0.deleted,
            episode_numbering_processed: value.0.episode_numbering_processed,
            favored: value.2.map(|f| f.favorite),
        }
    }
}

pub fn map_file_url_with_api_key(
    url: &Option<String>,
    remote_url: &str,
    api_key: &Option<String>,
) -> String {
    match url {
        Some(url) => {
            let mut url_encoded = PathBuf::from(url)
                .components()
                .map(|c| urlencoding::encode(c.as_os_str().to_str().unwrap()))
                .collect::<Vec<Cow<str>>>()
                .join("/");
            url_encoded = ENVIRONMENT_SERVICE.server_url.to_owned() + &url_encoded;

            match ENVIRONMENT_SERVICE.any_auth_enabled {
                true => match &api_key {
                    None => url_encoded,
                    Some(api_key) => url_encoded + "?apiKey=" + api_key,
                },
                false => url_encoded,
            }
        }
        None => remote_url.to_string(),
    }
}

pub fn map_file_url(url: &Option<String>, remote_url: &str, user: &Option<User>) -> String {
    match url {
        Some(url) => {
            let mut url_encoded = PathBuf::from(url)
                .components()
                .map(|c| urlencoding::encode(c.as_os_str().to_str().unwrap()))
                .collect::<Vec<Cow<str>>>()
                .join("/");
            url_encoded = ENVIRONMENT_SERVICE.server_url.to_owned() + &url_encoded;

            match ENVIRONMENT_SERVICE.any_auth_enabled {
                true => match &user {
                    None => url_encoded,
                    Some(user) => match &user.api_key {
                        None => url_encoded,
                        Some(key) => url_encoded + "?apiKey=" + key,
                    },
                },
                false => url_encoded,
            }
        }
        None => remote_url.to_string(),
    }
}
