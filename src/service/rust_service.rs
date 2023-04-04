use std::io::Error;
use crate::constants::constants::{PodcastType, ITUNES_URL};
use crate::db::DB;
use crate::models::itunes_models::Podcast;
use crate::models::messages::BroadcastMessage;
use crate::models::models::PodcastInsertModel;
use crate::models::web_socket_message::Lobby;
use crate::service::environment_service::EnvironmentService;
use crate::service::file_service::FileService;
use crate::service::mapping_service::MappingService;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use crate::unwrap_string;
use actix::Addr;
use actix_web::web::Data;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, ClientBuilder as AsyncClientBuilder};
use serde_json::Value;
use sha1::{Digest, Sha1};
use std::time::SystemTime;
use tokio::task::spawn_blocking;

#[derive(Clone)]
pub struct PodcastService {
    pub client: Client,
    pub podcast_episode_service: PodcastEpisodeService,
    pub db: DB,
    pub environment_service: EnvironmentService,
}

impl PodcastService {
    pub fn new() -> PodcastService {
        PodcastService {
            client: AsyncClientBuilder::new().build().unwrap(),
            db: DB::new().unwrap(),
            podcast_episode_service: PodcastEpisodeService::new(),
            environment_service: EnvironmentService::new(),
        }
    }

    pub async fn find_podcast(&mut self, podcast: &str) -> Value {
        let result = self
            .client
            .get(ITUNES_URL.to_owned() + podcast)
            .send()
            .await
            .unwrap();
        log::info!("Found podcast: {}", result.url());
        return result.json().await.unwrap();
    }

    pub async fn find_podcast_on_podindex(&mut self, podcast: &str) -> Value {
        let headers = self.compute_podindex_header();
        let result = self
            .client
            .get("https://api.podcastindex.org/api/1.0/search/byterm?q=".to_owned() + podcast)
            .headers(headers)
            .send()
            .await
            .unwrap();

        log::info!("Found podcast: {}", result.url());
        return result.json().await.unwrap();
    }

    pub async fn insert_podcast_from_podindex(&mut self, id: i32, lobby: Data<Addr<Lobby>>) {
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

        self.handle_insert_of_podcast(
            PodcastInsertModel {
                title: unwrap_string(&podcast["feed"]["title"]),
                id,
                feed_url: unwrap_string(&podcast["feed"]["url"]),
                image_url: unwrap_string(&podcast["feed"]["image"]),
            },
            mapping_service,
            lobby,
        )
        .await;
    }

    pub async fn handle_insert_of_podcast(
        &mut self,
        podcast_insert: PodcastInsertModel,
        mapping_service: MappingService,
        lobby: Data<Addr<Lobby>>,
    ) {
        let fileservice = FileService::new();
        let mut db = DB::new().unwrap();
        let inserted_podcast = db.add_podcast_to_database(
            podcast_insert.title,
            podcast_insert.id.to_string(),
            podcast_insert.feed_url,
            podcast_insert.image_url.clone(),
        );
        FileService::create_podcast_directory_exists(&podcast_insert.id.clone().to_string());
        fileservice
            .download_podcast_image(
                &podcast_insert.id.clone().to_string(),
                &podcast_insert.image_url.clone().to_string(),
            )
            .await;
        let podcast = db
            .get_podcast_by_track_id(podcast_insert.id.clone())
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
                    let mut podcast_service = PodcastService::new();
                    let mut podcast_episode_service = PodcastEpisodeService::new();
                    log::debug!("Inserting podcast episodes: {}", podcast.name);
                    let inserted_podcasts =
                        podcast_episode_service.insert_podcast_episodes(podcast.clone());

                    lobby.get_ref().do_send(BroadcastMessage {
                        podcast_episode: None,
                        type_of: PodcastType::AddPodcastEpisodes,
                        message: format!("Added podcast episodes: {}", podcast.name),
                        podcast: Option::from(podcast.clone()),
                        podcast_episodes: Option::from(inserted_podcasts),
                    });
                    podcast_service.schedule_episode_download(podcast, Some(lobby));
                })
                .await
                .unwrap();
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
    ) {
        let settings = self.db.get_settings();
        match settings {
            Some(settings) => {
                if settings.auto_download {
                    let result = self
                        .podcast_episode_service
                        .get_last_5_podcast_episodes(podcast.clone());
                    for podcast_episode in result {
                        self.podcast_episode_service
                            .download_podcast_episode_if_not_locally_available(
                                podcast_episode,
                                podcast.clone(),
                                lobby.clone(),
                            );
                    }
                }
            }
            None => {
                log::error!("Error getting settings");
            }
        }
    }

    pub fn refresh_podcast(&mut self, podcast: Podcast, lobby: Data<Addr<Lobby>>) {
        log::info!("Refreshing podcast: {}", podcast.name);
        self.podcast_episode_service
            .insert_podcast_episodes(podcast.clone());
        self.schedule_episode_download(podcast, Some(lobby));
    }

    pub fn update_favor_podcast(&mut self, id: i32, x: bool) {
        self.db.update_podcast_favor(&id, x).unwrap();
    }

    pub fn get_podcast_by_id(&mut self, id: i32) -> Podcast {
        self.db.get_podcast(id).unwrap()
    }

    pub fn get_favored_podcasts(&mut self) -> Vec<Podcast> {
        self.db.get_favored_podcasts().unwrap()
    }

    pub fn update_active_podcast(&mut self, id: i32) {
        self.db.update_podcast_active(id);
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

        headers.insert("User-Agent", HeaderValue::from_str("Podfetch").unwrap());
        headers.insert(
            "X-Auth-Key",
            HeaderValue::from_str(&self.environment_service.podindex_api_key.clone()).unwrap(),
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

    pub fn get_podcast(&mut self, podcast_id_to_be_searched: i32)->Result<Podcast, Error>{
        self.db.get_podcast(podcast_id_to_be_searched)
    }

    pub fn get_podcasts(&mut self) -> Result<Vec<Podcast>, String> {
        self.db.get_podcasts()
    }
}
