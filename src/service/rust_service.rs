use crate::constants::inner_constants::{
    PodcastType, COMMON_USER_AGENT, ENVIRONMENT_SERVICE, ITUNES_URL, MAIN_ROOM,
};
use crate::models::podcast_dto::PodcastDto;
use crate::models::podcasts::Podcast;

use crate::models::messages::BroadcastMessage;
use crate::models::misc_models::PodcastInsertModel;

use crate::adapters::api::models::podcast_episode_dto::PodcastEpisodeDto;
use crate::controllers::server::ChatServerHandle;
use crate::models::favorites::Favorite;
use crate::models::order_criteria::{OrderCriteria, OrderOption};
use crate::models::podcast_settings::PodcastSetting;
use crate::models::settings::Setting;
use crate::models::tag::Tag;
use crate::service::file_service::FileService;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use crate::unwrap_string;
use crate::utils::error::{map_reqwest_error, CustomError};
use crate::utils::http_client::get_http_client;
use actix_web::web::Data;
use reqwest::header::{HeaderMap, HeaderValue};
use rss::Channel;
use serde::Serialize;
use serde_json::Value;
use sha1::{Digest, Sha1};
use std::time::SystemTime;
use tokio::task::spawn_blocking;

pub struct PodcastService;

impl PodcastService {
    pub async fn find_podcast(podcast: &str) -> Value {
        let query = vec![("term", podcast), ("entity", "podcast")];
        let result = get_http_client()
            .get(ITUNES_URL)
            .query(&query)
            .send()
            .await
            .unwrap();
        log::info!("Found podcast: {}", result.url());
        let res_of_search = result.json().await;

        if let Ok(res) = res_of_search {
            res
        } else {
            log::error!(
                "Error searching for podcast: {}",
                res_of_search.err().unwrap()
            );
            serde_json::from_str("{}").unwrap()
        }
    }

    pub async fn find_podcast_on_podindex(podcast: &str) -> Result<Value, CustomError> {
        let headers = Self::compute_podindex_header();

        let query = vec![("q", podcast)];

        let result = get_http_client()
            .get("https://api.podcastindex.org/api/1.0/search/byterm")
            .query(&query)
            .headers(headers)
            .send()
            .await
            .map_err(map_reqwest_error)?;

        log::info!("Found podcast: {}", result.url());

        let status = result.status();
        let possible_json = result.text().await.map_err(map_reqwest_error)?;

        if status.is_client_error() || status.is_server_error() {
            log::error!("Error searching for podcast: {}", possible_json);
            Err(CustomError::BadRequest(possible_json))
        } else {
            let res_of_search = serde_json::from_str(&possible_json);

            if let Ok(res) = res_of_search {
                Ok(res)
            } else {
                log::error!(
                    "Error searching for podcast: {}",
                    res_of_search.err().unwrap()
                );
                Ok(serde_json::from_str("{}").unwrap())
            }
        }
    }

    pub async fn insert_podcast_from_podindex(
        id: i32,
        lobby: Data<ChatServerHandle>,
    ) -> Result<Podcast, CustomError> {
        let resp = get_http_client()
            .get(
                "https://api.podcastindex.org/api/1.0/podcasts/byfeedid?id=".to_owned()
                    + &id.to_string(),
            )
            .headers(Self::compute_podindex_header())
            .send()
            .await
            .unwrap();

        println!("Result: {:?}", resp);

        let podcast = resp.json::<Value>().await.unwrap();

        Self::handle_insert_of_podcast(
            PodcastInsertModel {
                title: unwrap_string(&podcast["feed"]["title"]),
                id,
                feed_url: unwrap_string(&podcast["feed"]["url"]),
                image_url: unwrap_string(&podcast["feed"]["image"]),
            },
            lobby,
            None,
        )
        .await
    }

    pub async fn handle_insert_of_podcast(
        podcast_insert: PodcastInsertModel,
        lobby: Data<ChatServerHandle>,
        channel: Option<Channel>,
    ) -> Result<Podcast, CustomError> {
        let opt_podcast = Podcast::find_by_rss_feed_url(&podcast_insert.feed_url.clone());
        if opt_podcast.is_some() {
            return Err(CustomError::Conflict(format!(
                "Podcast with feed url {} already exists",
                podcast_insert.feed_url
            )));
        }

        let podcast_directory_created =
            FileService::create_podcast_directory_exists(&podcast_insert, channel).await?;

        let inserted_podcast = Podcast::add_podcast_to_database(
            podcast_insert.title,
            podcast_insert.id.to_string(),
            podcast_insert.feed_url,
            podcast_insert.image_url.clone(),
            podcast_directory_created,
        )?;

        FileService::download_podcast_image(
            &inserted_podcast.directory_name.clone().to_string(),
            podcast_insert.image_url.clone().to_string(),
            &podcast_insert.id.clone().to_string(),
        )
        .await;
        let podcast = Podcast::get_podcast_by_track_id(podcast_insert.id).unwrap();
        lobby
            .send_broadcast(
                MAIN_ROOM.parse().unwrap(),
                serde_json::to_string(&BroadcastMessage {
                    podcast_episode: None,
                    type_of: PodcastType::AddPodcast,
                    message: format!("Added podcast: {}", inserted_podcast.name),
                    podcast: podcast.clone().map(|p| p.into()),
                    podcast_episodes: None,
                })
                .unwrap(),
            )
            .await;
        match podcast {
            Some(podcast) => {
                spawn_blocking(move || {
                    log::debug!("Inserting podcast episodes of {}", podcast.name);
                    let inserted_podcasts =
                        PodcastEpisodeService::insert_podcast_episodes(podcast.clone()).unwrap();

                    lobby.send_broadcast_sync(
                        MAIN_ROOM.parse().unwrap(),
                        serde_json::to_string(&BroadcastMessage {
                            podcast_episode: None,
                            type_of: PodcastType::AddPodcastEpisodes,
                            message: format!("Added podcast episodes: {}", podcast.name),
                            podcast: Some(podcast.clone().into()),
                            podcast_episodes: Option::from(
                                inserted_podcasts
                                    .into_iter()
                                    .map(|p| p.into())
                                    .collect::<Vec<PodcastEpisodeDto>>(),
                            ),
                        })
                        .unwrap(),
                    );
                    if let Err(e) = Self::schedule_episode_download(podcast, Some(lobby)) {
                        log::error!("Error scheduling episode download: {}", e);
                    }
                })
                .await
                .unwrap();
                Ok(inserted_podcast)
            }
            None => {
                panic!("No podcast found")
            }
        }
    }

    pub fn schedule_episode_download(
        podcast: Podcast,
        lobby: Option<Data<ChatServerHandle>>,
    ) -> Result<(), CustomError> {
        let settings = Setting::get_settings()?;
        let podcast_settings = PodcastSetting::get_settings(podcast.id)?;
        match settings {
            Some(settings) => {
                if (podcast_settings.is_some() && podcast_settings.unwrap().auto_download)
                    || settings.auto_download
                {
                    let result =
                        PodcastEpisodeService::get_last_n_podcast_episodes(podcast.clone())?;
                    for podcast_episode in result {
                        if !podcast_episode.deleted {
                            if let Err(e) =
                            PodcastEpisodeService::download_podcast_episode_if_not_locally_available(
                                    podcast_episode,
                                    podcast.clone(),
                                    lobby.clone(),
                                ){
                                log::error!("Error downloading podcast episode: {}", e);
                            }
                        }
                    }
                }
                Ok(())
            }
            None => {
                log::error!("Error getting settings");
                Err(CustomError::Unknown)
            }
        }
    }

    pub fn refresh_podcast(
        podcast: Podcast,
        lobby: Data<ChatServerHandle>,
    ) -> Result<(), CustomError> {
        log::info!("Refreshing podcast: {}", podcast.name);
        PodcastEpisodeService::insert_podcast_episodes(podcast.clone())?;
        Self::schedule_episode_download(podcast.clone(), Some(lobby.clone()))
    }

    pub fn update_favor_podcast(id: i32, x: bool, username: &str) -> Result<(), CustomError> {
        Favorite::update_podcast_favor(&id, x, username)
    }

    pub fn get_podcast_by_id(id: i32) -> Podcast {
        Podcast::get_podcast(id).unwrap()
    }

    pub fn get_favored_podcasts(found_username: String) -> Result<Vec<PodcastDto>, CustomError> {
        Favorite::get_favored_podcasts(found_username)
    }

    pub fn update_active_podcast(id: i32) -> Result<(), CustomError> {
        Podcast::update_podcast_active(id)
    }

    fn compute_podindex_header() -> HeaderMap {
        let seconds = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut headers = HeaderMap::new();
        let non_hashed_string = ENVIRONMENT_SERVICE.podindex_api_key.clone().to_owned()
            + &*ENVIRONMENT_SERVICE.podindex_api_secret.clone()
            + &seconds.to_string();
        let mut hasher = Sha1::new();

        hasher.update(non_hashed_string);

        let hashed_auth_key = format!("{:x}", hasher.finalize());

        headers.insert(
            "User-Agent",
            HeaderValue::from_str(COMMON_USER_AGENT).unwrap(),
        );
        headers.insert(
            "X-Auth-Key",
            HeaderValue::from_str(&ENVIRONMENT_SERVICE.podindex_api_key).unwrap(),
        );
        headers.insert(
            "X-Auth-Date",
            HeaderValue::from_str(&seconds.to_string()).unwrap(),
        );
        headers.insert(
            "Authorization",
            HeaderValue::from_str(&hashed_auth_key).unwrap(),
        );

        headers
    }

    pub fn get_podcast(podcast_id_to_be_searched: i32) -> Result<Podcast, CustomError> {
        Podcast::get_podcast(podcast_id_to_be_searched)
    }

    pub fn get_podcasts(u: &str) -> Result<Vec<PodcastDto>, CustomError> {
        Podcast::get_podcasts(u)
    }

    pub fn search_podcasts_favored(
        order: OrderCriteria,
        title: Option<String>,
        latest_pub: OrderOption,
        designated_username: String,
        tag: Option<String>,
    ) -> Result<Vec<impl Serialize>, CustomError> {
        let podcasts =
            Favorite::search_podcasts_favored(order, title, latest_pub, &designated_username)?;
        let mut podcast_dto_vec: Vec<PodcastDto> = Vec::new();
        for podcast in podcasts {
            let tags_of_podcast = Tag::get_tags_of_podcast(podcast.0.id, &designated_username)?;
            podcast_dto_vec.push(
                (
                    podcast.0.clone(),
                    Some(podcast.1.clone()),
                    tags_of_podcast.clone(),
                )
                    .into(),
            );
        }

        if let Some(tag) = tag {
            let found_tag = Tag::get_tag_by_id_and_username(&tag, &designated_username)?;

            if let Some(foud_tag) = found_tag {
                podcast_dto_vec = podcast_dto_vec
                    .into_iter()
                    .filter(|p| p.tags.iter().any(|t| t.id == foud_tag.id))
                    .collect::<Vec<PodcastDto>>()
            }
        }

        Ok(podcast_dto_vec)
    }

    pub fn search_podcasts(
        order: OrderCriteria,
        title: Option<String>,
        latest_pub: OrderOption,
        designated_username: String,
        tag: Option<String>,
    ) -> Result<Vec<PodcastDto>, CustomError> {
        let podcasts = Favorite::search_podcasts(order, title, latest_pub, &designated_username)?;
        let mut mapped_result = podcasts
            .iter()
            .map(|podcast| {
                let tags = Tag::get_tags_of_podcast(podcast.0.id, &designated_username).unwrap();
                (podcast.0.clone(), podcast.1.clone(), tags).into()
            })
            .collect::<Vec<PodcastDto>>();

        if let Some(tag) = tag {
            let found_tag = Tag::get_tag_by_id_and_username(&tag, &designated_username)?;

            if let Some(foud_tag) = found_tag {
                mapped_result = mapped_result
                    .into_iter()
                    .filter(|p| p.tags.iter().any(|t| t.id == foud_tag.id))
                    .collect::<Vec<PodcastDto>>()
            }
        }
        Ok(mapped_result)
    }
}
