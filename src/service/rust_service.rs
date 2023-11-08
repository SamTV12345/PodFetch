
use std::sync::{MutexGuard};
use crate::constants::inner_constants::{PodcastType, ITUNES_URL, COMMON_USER_AGENT};
use crate::models::podcast_dto::PodcastDto;
use crate::models::podcasts::Podcast;

use crate::models::messages::BroadcastMessage;
use crate::models::misc_models::PodcastInsertModel;
use crate::models::web_socket_message::Lobby;
use crate::service::environment_service::EnvironmentService;
use crate::service::file_service::FileService;
use crate::service::mapping_service::MappingService;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use crate::{DBType as DBConnection, unwrap_string};
use actix::Addr;
use actix_web::web::Data;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, ClientBuilder as AsyncClientBuilder};
use serde_json::Value;
use sha1::{Digest, Sha1};
use std::time::SystemTime;

use serde::Serialize;
use tokio::task::spawn_blocking;
use crate::config::dbconfig::establish_connection;

use crate::models::favorites::Favorite;
use crate::models::order_criteria::{OrderCriteria, OrderOption};
use crate::models::settings::Setting;
use crate::utils::error::{CustomError, map_reqwest_error};
use crate::DBType as DbConnection;

#[derive(Clone)]
pub struct PodcastService {
    pub client: Client,
    pub podcast_episode_service: PodcastEpisodeService,
    pub environment_service: EnvironmentService,
}

impl Default for PodcastService {
    fn default() -> Self {
        Self::new()
    }
}

impl PodcastService {
    pub fn new() -> PodcastService {
        PodcastService {
            client: AsyncClientBuilder::new().build().unwrap(),
            podcast_episode_service: PodcastEpisodeService::new(),
            environment_service: EnvironmentService::new(),
        }
    }

    pub async fn find_podcast(&mut self, podcast: &str) -> Value {
        let query = vec![("term", podcast), ("entity", "podcast")];
        let result = self
            .client
            .get(ITUNES_URL)
            .query(&query)
            .send()
            .await
            .unwrap();
        log::info!("Found podcast: {}", result.url());
        let res_of_search =  result.json().await;

        if let Ok(res) = res_of_search {
            res
        }
        else {
            log::error!("Error searching for podcast: {}", res_of_search.err().unwrap());
            serde_json::from_str("{}").unwrap()
        }
    }

    pub async fn find_podcast_on_podindex(&mut self, podcast: &str) -> Result<Value, CustomError> {
        let headers = self.compute_podindex_header();

        let query = vec![
            ("q", podcast)
        ];

        let result = self
            .client
            .get("https://api.podcastindex.org/api/1.0/search/byterm")
            .query(&query)
            .headers(headers)
            .send()
            .await
            .map_err(map_reqwest_error)?;

        log::info!("Found podcast: {}", result.url());
        result.json().await.map_err(map_reqwest_error)
    }

    pub async fn insert_podcast_from_podindex(&mut self, conn: &mut DbConnection, id: i32,
                                              lobby: Data<Addr<Lobby>>) ->Result<Podcast,
        CustomError>{
        let mapping_service = MappingService::new();
        let resp = self
            .client
            .get(
                "https://api.podcastindex.org/api/1.0/podcasts/byfeedid?id=".to_owned()
                    + &id.to_string(),
            )
            .headers(self.compute_podindex_header())
            .send()
            .await
            .unwrap();

        println!("Result: {:?}", resp);

        let podcast = resp.json::<Value>().await.unwrap();

        self.handle_insert_of_podcast(conn,
                                      PodcastInsertModel {
                                          title: unwrap_string(&podcast["feed"]["title"]),
                                          id,
                                          feed_url: unwrap_string(&podcast["feed"]["url"]),
                                          image_url: unwrap_string(&podcast["feed"]["image"]),
                                      },
                                      mapping_service,
                                      lobby,
        )
            .await
    }

    pub async fn handle_insert_of_podcast(
        &mut self,
        conn: &mut DbConnection,
        podcast_insert: PodcastInsertModel,
        mapping_service: MappingService,
        lobby: Data<Addr<Lobby>>) ->Result<Podcast,CustomError>{
        let opt_podcast = Podcast::find_by_rss_feed_url(conn, &podcast_insert.feed_url.clone());
        if opt_podcast.is_some() {
            return Err(CustomError::Conflict(format!("Podcast with feed url {} already exists", podcast_insert.feed_url)));
        }

        let fileservice = FileService::new();

        let podcast_directory_created = FileService::create_podcast_directory_exists(
            &podcast_insert.title.clone(), &podcast_insert.id.clone().to_string(), conn)?;

        let inserted_podcast = Podcast::add_podcast_to_database(
            conn,
            podcast_insert.title,
            podcast_insert.id.to_string(),
            podcast_insert.feed_url,
            podcast_insert.image_url.clone(),
            podcast_directory_created
        )?;

        fileservice
            .download_podcast_image(&inserted_podcast.directory_name.clone().to_string(),
                                    podcast_insert.image_url.clone().to_string(),
                                    &podcast_insert.id.clone().to_string(), conn)
            .await;
        let podcast = Podcast::get_podcast_by_track_id(conn, podcast_insert.id)
            .unwrap();
        lobby
            .get_ref()
            .send(BroadcastMessage {
                podcast_episode: None,
                type_of: PodcastType::AddPodcast,
                message: format!("Added podcast: {}", inserted_podcast.name),
                podcast: Option::from(
                    mapping_service.map_podcast_to_podcast_dto(&podcast.clone().unwrap()),
                ),
                podcast_episodes: None,
            })
            .await
            .unwrap();
        match podcast {
            Some(podcast) => {
                spawn_blocking(move || {
                    let mut conn = establish_connection();
                    let mut podcast_service = PodcastService::new();
                    let mut podcast_episode_service = PodcastEpisodeService::new();
                    log::debug!("Inserting podcast episodes of {}", podcast.name);
                    let inserted_podcasts =
                        podcast_episode_service.insert_podcast_episodes(&mut conn, podcast.clone()).unwrap();

                    lobby.get_ref().do_send(BroadcastMessage {
                        podcast_episode: None,
                        type_of: PodcastType::AddPodcastEpisodes,
                        message: format!("Added podcast episodes: {}", podcast.name),
                        podcast: Option::from(podcast.clone()),
                        podcast_episodes: Option::from(inserted_podcasts),
                    });
                    podcast_service.schedule_episode_download(podcast, Some(lobby),&mut conn).unwrap();
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
        &mut self,
        podcast: Podcast,
        lobby: Option<Data<Addr<Lobby>>>,
        conn: &mut DbConnection
    ) ->Result<(),CustomError> {
        let settings = Setting::get_settings(conn)?;
        match settings {
            Some(settings) => {
                if settings.auto_download {
                    let result = PodcastEpisodeService::get_last_n_podcast_episodes(conn, podcast.clone())?;
                    for podcast_episode in result {
                        if !podcast_episode.deleted {
                            self.podcast_episode_service
                                .download_podcast_episode_if_not_locally_available(
                                    podcast_episode,
                                    podcast.clone(),
                                    lobby.clone(),
                                    conn
                                )?;
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

    pub fn refresh_podcast(&mut self, podcast: Podcast, lobby: Data<Addr<Lobby>>, conn:&mut
    DbConnection) ->Result<(),CustomError> {
        log::info!("Refreshing podcast: {}", podcast.name);
        self.podcast_episode_service
            .insert_podcast_episodes(conn, podcast.clone())?;
        self.schedule_episode_download(podcast.clone(), Some(lobby.clone()), conn)
    }

    pub fn update_favor_podcast(&mut self, id: i32, x: bool, username: String, conn: &mut
    DbConnection) -> Result<(),CustomError> {
        Favorite::update_podcast_favor(&id, x, conn,username)
    }

    pub fn get_podcast_by_id(&mut self,conn: &mut DbConnection, id: i32) -> Podcast {
        Podcast::get_podcast(conn,id).unwrap()
    }

    pub fn get_favored_podcasts(&mut self, found_username: String,
                                mapping_service:MappingService, conn: &mut
        DbConnection) ->
                                Result<Vec<PodcastDto>, CustomError> {
        Favorite::get_favored_podcasts(found_username,conn,mapping_service)
    }

    pub fn update_active_podcast(conn: &mut DbConnection, id: i32) -> Result<(),CustomError> {
        Podcast::update_podcast_active(conn, id)
    }

    fn compute_podindex_header(&mut self) -> HeaderMap {
        let seconds = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut headers = HeaderMap::new();
        let non_hashed_string = self.environment_service.podindex_api_key.clone().to_owned()
            + &*self.environment_service.podindex_api_secret.clone()
            + &seconds.to_string();
        let mut hasher = Sha1::new();

        hasher.update(non_hashed_string);

        let hashed_auth_key = format!("{:x}", hasher.finalize());

        headers.insert("User-Agent", HeaderValue::from_str(COMMON_USER_AGENT).unwrap());
        headers.insert(
            "X-Auth-Key",
            HeaderValue::from_str(&self.environment_service.podindex_api_key).unwrap(),
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

    pub fn get_podcast(conn: &mut DbConnection, podcast_id_to_be_searched: i32)->Result<Podcast,
        CustomError>{
        Podcast::get_podcast(conn, podcast_id_to_be_searched)
    }

    pub fn get_podcasts(conn: &mut DbConnection, u: String, mapping_service: MutexGuard<MappingService>) ->
    Result<Vec<PodcastDto>, CustomError> {
        Podcast::get_podcasts(conn, u, mapping_service)
    }

    pub fn search_podcasts_favored(
        &mut self, order:OrderCriteria, title: Option<String>, latest_pub: OrderOption,
        mapping_service: MutexGuard<MappingService>, conn: &mut
        DbConnection,
        designated_username: String) -> Result<Vec<impl Serialize>, CustomError>{
        let podcasts = Favorite::search_podcasts_favored(conn, order, title, latest_pub,
                                                         designated_username)?;
        let mut podcast_dto_vec = Vec::new();
        for podcast in podcasts {
            let podcast_dto = mapping_service.map_podcast_to_podcast_dto_with_favorites_option(&podcast);
            podcast_dto_vec.push(podcast_dto);
        }
        Ok(podcast_dto_vec)
    }


    pub fn search_podcasts(&mut self, order:OrderCriteria, mapping_service:
    MutexGuard<MappingService>,title:
                           Option<String>, latest_pub: OrderOption,conn: &mut
    DbConnection,
                           designated_username: String) -> Result<Vec<PodcastDto>, CustomError>{

        let podcasts = Favorite::search_podcasts(conn, order, title, latest_pub,
                                                 designated_username)?;
        let mapped_result = podcasts
            .iter()
            .map(|podcast| mapping_service.map_podcast_to_podcast_dto_with_favorites
            (podcast))
            .collect::<Vec<PodcastDto>>();
        Ok(mapped_result)
    }
}
