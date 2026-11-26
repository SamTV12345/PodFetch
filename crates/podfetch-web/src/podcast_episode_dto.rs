use crate::url_rewriting::resolve_image_url;
use chrono::NaiveDateTime;
use common_infrastructure::config::FileHandlerType;
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
use podfetch_library::domain::favorite_podcast_episode::FavoritePodcastEpisode;
use podfetch_persistence::podcast_episode::PodcastEpisodeEntity as PodcastEpisode;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::path::PathBuf;
use std::str::FromStr;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct PodcastEpisodeDto {
    pub id: String,
    #[serde(rename = "legacyId")]
    pub legacy_id: Option<i64>,
    pub podcast_id: String,
    pub episode_id: String,
    pub name: String,
    pub url: String,
    pub date_of_recording: String,
    pub image_url: String,
    pub total_time: i32,
    pub local_url: String,
    pub local_image_url: String,
    pub description: String,
    pub status: bool,
    pub download_time: Option<NaiveDateTime>,
    pub guid: String,
    pub deleted: bool,
    pub episode_numbering_processed: bool,
    pub favored: Option<bool>,
}

pub enum FileType {
    Image,
    Episode,
}

impl PodcastEpisodeDto {
    /// `api_key` is the caller's own key; local file and proxy URLs carry it
    /// when authentication is enabled so players can fetch them directly.
    pub fn from_episode_with_user(
        episode: PodcastEpisode,
        api_key: Option<&str>,
        favorite: Option<FavoritePodcastEpisode>,
        server_url: &str,
    ) -> Self {
        let local_url = map_url(
            &episode,
            &episode.file_episode_path,
            &episode.url,
            api_key,
            FileType::Episode,
            server_url,
        );
        let local_image_url = resolve_image_url(
            &map_url(
                &episode,
                &episode.file_image_path,
                &episode.image_url,
                api_key,
                FileType::Image,
                server_url,
            ),
            server_url,
        );
        PodcastEpisodeDto {
            id: episode.id.clone(),
            legacy_id: episode.legacy_id,
            podcast_id: episode.podcast_id.clone(),
            episode_id: episode.episode_id.clone(),
            name: episode.name.clone(),
            url: episode.url.clone(),
            date_of_recording: episode.date_of_recording.clone(),
            image_url: episode.image_url.clone(),
            total_time: episode.total_time,
            local_url,
            local_image_url,
            description: episode.description.clone(),
            download_time: episode.download_time,
            guid: episode.guid.clone(),
            deleted: episode.deleted,
            episode_numbering_processed: episode.episode_numbering_processed,
            favored: favorite.map(|f| f.favorite),
            status: episode.is_downloaded(),
        }
    }

    pub fn from_episode_with_api_key(
        episode: PodcastEpisode,
        api_key: Option<String>,
        favorite: Option<FavoritePodcastEpisode>,
        server_url: &str,
    ) -> Self {
        let local_url = map_file_url_with_api_key(
            &episode,
            &episode.file_episode_path,
            &episode.url,
            &api_key,
            server_url,
        );
        let local_image_url = resolve_image_url(
            &map_file_url_with_api_key(
                &episode,
                &episode.file_image_path,
                &episode.image_url,
                &api_key,
                server_url,
            ),
            server_url,
        );
        PodcastEpisodeDto {
            id: episode.id.clone(),
            legacy_id: episode.legacy_id,
            podcast_id: episode.podcast_id.clone(),
            episode_id: episode.episode_id.clone(),
            name: episode.name.clone(),
            url: episode.url.clone(),
            date_of_recording: episode.date_of_recording.clone(),
            image_url: episode.image_url.clone(),
            total_time: episode.total_time,
            local_url,
            local_image_url,
            description: episode.description.clone(),
            download_time: episode.download_time,
            guid: episode.guid.clone(),
            deleted: episode.deleted,
            episode_numbering_processed: episode.episode_numbering_processed,
            favored: favorite.map(|f| f.favorite),
            status: episode.is_downloaded(),
        }
    }
}

fn map_file_url_with_api_key(
    podcast_episode: &PodcastEpisode,
    local_url: &Option<String>,
    remote_url: &str,
    api_key: &Option<String>,
    server_url: &str,
) -> String {
    match &podcast_episode.download_location {
        Some(location) => {
            let handle = FileHandlerType::from_stored(location);
            match handle {
                FileHandlerType::Local => {
                    map_local_file_url_with_api_key(local_url, remote_url, api_key, server_url)
                }
                FileHandlerType::S3 => map_s3_url(local_url, remote_url),
            }
        }
        None => remote_url.to_string(),
    }
}

pub fn map_local_file_url_with_api_key(
    url: &Option<String>,
    remote_url: &str,
    api_key: &Option<String>,
    server_url: &str,
) -> String {
    match url {
        Some(url) => {
            let mut url_encoded = PathBuf::from(url)
                .components()
                .map(|c| urlencoding::encode(c.as_os_str().to_str().unwrap()))
                .collect::<Vec<Cow<str>>>()
                .join("/");
            let urlencoded = url_encoded.clone();
            url_encoded = server_url.to_owned();
            url_encoded.push_str(&urlencoded);

            if ENVIRONMENT_SERVICE.any_auth_enabled {
                match &api_key {
                    None => url_encoded,
                    Some(api_key) => format!("{}{}{}", url_encoded, "?apiKey=", api_key),
                }
            } else {
                url_encoded
            }
        }
        None => remote_url.to_string(),
    }
}

fn map_url(
    episode: &PodcastEpisode,
    local_url: &Option<String>,
    remote_url: &str,
    api_key: Option<&str>,
    r#type: FileType,
    server_url: &str,
) -> String {
    match &episode.download_location {
        Some(location) => {
            let handle = FileHandlerType::from_stored(location);
            match handle {
                FileHandlerType::Local => map_file_url(local_url, remote_url, api_key, server_url),
                FileHandlerType::S3 => map_s3_url(local_url, remote_url),
            }
        }
        None => match r#type {
            FileType::Image => remote_url.to_string(),
            FileType::Episode => {
                if server_url.is_empty() {
                    return remote_url.to_string();
                }
                let mut url = url::Url::from_str(&format!("{server_url}proxy/podcast")).unwrap();
                if ENVIRONMENT_SERVICE.any_auth_enabled
                    && let Some(key) = api_key
                {
                    url.query_pairs_mut().append_pair("apiKey", key);
                }
                url.query_pairs_mut()
                    .append_pair("episodeId", &episode.episode_id);
                url.to_string()
            }
        },
    }
}

pub fn map_file_url(
    url: &Option<String>,
    remote_url: &str,
    api_key: Option<&str>,
    server_url: &str,
) -> String {
    match url {
        Some(url) => {
            let mut url_encoded = PathBuf::from(url)
                .components()
                .map(|c| urlencoding::encode(c.as_os_str().to_str().unwrap()))
                .collect::<Vec<Cow<str>>>()
                .join("/");
            url_encoded = format!("{server_url}{url_encoded}");

            match api_key {
                Some(key) if ENVIRONMENT_SERVICE.any_auth_enabled => {
                    format!("{url_encoded}?apiKey={key}")
                }
                _ => url_encoded,
            }
        }
        None => remote_url.to_string(),
    }
}

pub fn map_s3_url(url: &Option<String>, remote_url: &str) -> String {
    match url {
        Some(url) => {
            let mut url_encoded = PathBuf::from(url)
                .components()
                .map(|c| urlencoding::encode(c.as_os_str().to_str().unwrap()))
                .collect::<Vec<Cow<str>>>()
                .join("/");
            url_encoded = format!("{}/{}", ENVIRONMENT_SERVICE.s3_config.endpoint, url_encoded);
            url_encoded
        }
        None => remote_url.to_string(),
    }
}
