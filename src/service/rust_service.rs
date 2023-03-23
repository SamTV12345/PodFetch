use actix::Addr;
use actix_web::web::Data;
use crate::constants::constants::{ITUNES_URL};
use reqwest::{Client, ClientBuilder as AsyncClientBuilder};
use serde_json::Value;
use crate::db::DB;
use crate::models::itunes_models::Podcast;
use crate::models::web_socket_message::Lobby;
use crate::service::podcast_episode_service::PodcastEpisodeService;

#[derive(Clone)]
pub struct PodcastService {
    pub client: Client,
    pub podcast_episode_service: PodcastEpisodeService,
    pub db: DB,
}

impl PodcastService {
    pub fn new() -> PodcastService {
        PodcastService {
            client: AsyncClientBuilder::new().build().unwrap(),
            db: DB::new().unwrap(),
            podcast_episode_service: PodcastEpisodeService::new()
        }
    }

    pub async fn find_podcast(&mut self,podcast: &str) -> Value {
        let result = self.client.get(ITUNES_URL.to_owned() + podcast).send().await.unwrap();
        log::info!("Found podcast: {}", result.url());
        return result.json().await.unwrap();
    }


    pub fn schedule_episode_download(&mut self,podcast: Podcast, lobby: Option<Data<Addr<Lobby>>>) {
        let result = self.podcast_episode_service.get_last_5_podcast_episodes(podcast.clone());
        for podcast_episode in result {
            self.podcast_episode_service.download_podcast_episode_if_not_locally_available(podcast_episode,
                                                                              podcast.clone(),
                                                                              lobby.clone());
        }
    }

    pub fn refresh_podcast(&mut self,podcast: Podcast, lobby: Data<Addr<Lobby>>) {
        log::info!("Refreshing podcast: {}", podcast.name);
        PodcastEpisodeService::insert_podcast_episodes(podcast.clone());
        self.schedule_episode_download(podcast, Some(lobby));
    }

    pub fn update_favor_podcast(&mut self, id:i32, x: bool){
        self.db.update_podcast_favor(&id, x).unwrap();
    }

    pub fn get_podcast_by_id(&mut self, id: i32) -> Podcast {
        self.db.get_podcast(id).unwrap()
    }

    pub fn get_favored_podcasts(&mut self)-> Vec<Podcast>{
        self.db.get_favored_podcasts().unwrap()
    }
}
