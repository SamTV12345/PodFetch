use std::sync::Mutex;
use std::thread;
use actix::Addr;
use actix_web::{HttpResponse, Responder, web};
use serde_json::{from_str, Value};
use crate::db::DB;
use crate::service::mapping_service::MappingService;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use actix_web::{get, post, put};
use actix_web::web::Data;
use crate::models::models::PodCastAddModel;
use crate::service::file_service::FileService;
use crate::unwrap_string;
use reqwest::{ClientBuilder as AsyncClientBuilder};
use tokio::task::spawn_blocking;
use crate::constants::constants::{PodcastType};
use crate::models::dto_models::PodcastFavorUpdateModel;
use crate::models::messages::BroadcastMessage;
use crate::models::web_socket_message::Lobby;
use crate::service::rust_service::PodcastService;


#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Find a podcast by its collection id", body = [Podcast])
),
tag="podcasts"
)]
#[get("/podcast/{id}")]
    pub async fn find_podcast_by_id( id: web::Path<String>, db: Data<Mutex<DB>>, mapping_service: Data<Mutex<MappingService>>) -> impl Responder {
        let id_num = from_str::<i32>(&id).unwrap();
        let podcast = db.lock().expect("Error acquiring lock").get_podcast(id_num)
            .expect("Error getting podcast");
        let mapping_service = mapping_service.lock().expect("Error acquiring lock");
        let mapped_podcast = mapping_service.map_podcast_to_podcast_dto(&podcast);
        HttpResponse::Ok().json(mapped_podcast)
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets all stored podcasts as a list", body = [Podcast])
),
tag="podcasts"
)]
#[get("/podcasts")]
pub async fn find_all_podcasts(db: Data<Mutex<DB>>, mapping_service:Data<Mutex<MappingService>>) -> impl Responder {

    let mapping_service = mapping_service.lock().expect("Error acquiring lock");
    let podcasts = db.lock().expect("Error acquiring lock").get_podcasts().unwrap();

    let mapped_podcasts = podcasts
        .into_iter()
        .map(|podcast| mapping_service.map_podcast_to_podcast_dto(&podcast)).collect::<Vec<_>>();
    HttpResponse::Ok().json(mapped_podcasts)
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Finds a podcast from the itunes url.", body = [ItunesModel])
),
tag="podcasts"
)]
#[get("/podcasts/{podcast}/search")]
pub async fn find_podcast(podcast: web::Path<String>, podcast_service: Data<Mutex<PodcastService>>) -> impl Responder {
    let mut podcast_service = podcast_service.lock().expect("Error locking podcastservice");
    log::debug!("Searching for podcast: {}", podcast);
    let res = podcast_service.find_podcast(&podcast);
    HttpResponse::Ok().json(res.await)
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Adds a podcast to the database.")),
tag="podcasts"
)]
#[post("/podcast")]
pub async fn add_podcast(track_id: web::Json<PodCastAddModel>,
                         lobby: Data<Addr<Lobby>>, db: Data<Mutex<DB>>, mapping_service: Data<Mutex<MappingService>>, fileservice: Data<Mutex<FileService>> ) ->
                                                                                             impl
Responder {
    let mapping_service = mapping_service.lock().expect("Error locking mapping service");
    let mut db = db.lock().expect("Error acquiring lock");
    let fileservice = fileservice.lock().expect("Error acquiring lock");
    let client = AsyncClientBuilder::new().build().unwrap();
    let res = client.get("https://itunes.apple.com/lookup?id=".to_owned()+&track_id
        .track_id
        .to_string())
        .send().await.unwrap();

    let res  = res.json::<Value>().await.unwrap();


    let inserted_podcast = db.add_podcast_to_database(unwrap_string
                                                     (&res["results"][0]["collectionName"]),
                               unwrap_string(&res["results"][0]["collectionId"]),
                               unwrap_string(&res["results"][0]["feedUrl"]),
                               unwrap_string(&res["results"][0]["artworkUrl600"]));
    FileService::create_podcast_directory_exists(&unwrap_string
        (&res["results"][0]["collectionId"]));
    fileservice.download_podcast_image(&unwrap_string(&res["results"][0]["collectionId"]),
                                        &unwrap_string(&res["results"][0]["artworkUrl600"])).await;
    let podcast = db.get_podcast_by_track_id(track_id.track_id).unwrap();
    lobby.get_ref()
        .send(
        BroadcastMessage{
            podcast_episode: None,
            type_of: PodcastType::AddPodcast,
            message: format!("Added podcast: {}", inserted_podcast.name),
            podcast: Option::from(mapping_service.map_podcast_to_podcast_dto(&podcast.clone()
                .unwrap())),
            podcast_episodes: None,
        }).await.unwrap();
    match podcast {
        Some(podcast) => {
            spawn_blocking(move || {
                let mut podcast_service = PodcastService::new();
                let mut podcast_episode_service = PodcastEpisodeService::new();
                log::debug!("Inserting podcast episodes: {}", podcast.name);
                let inserted_podcasts = podcast_episode_service.insert_podcast_episodes(podcast
                    .clone());

                lobby.get_ref().do_send(BroadcastMessage {
                    podcast_episode: None,
                    type_of: PodcastType::AddPodcastEpisodes,
                    message: format!("Added podcast episodes: {}", podcast.name),
                    podcast: Option::from(podcast.clone()),
                    podcast_episodes: Option::from(inserted_podcasts),
                });
                podcast_service.schedule_episode_download(podcast, Some(lobby));
            }).await.unwrap();
        },
        None => {panic!("No podcast found")}
    }
    log::info!("Added podcast: {}", unwrap_string(&res["results"][0]["collectionName"]));
    HttpResponse::Ok()
}


#[get("/podcasts/{podcast}/query")]
pub async fn query_for_podcast(podcast: web::Path<String>, podcast_service: Data<Mutex<PodcastEpisodeService>>) -> impl Responder {
    let mut podcast_service = podcast_service.lock().unwrap();
    let res = podcast_service.query_for_podcast(&podcast);

    HttpResponse::Ok().json(res)
}

#[post("/podcast/{id}/refresh")]
pub async fn download_podcast(id: web::Path<String>, lobby: Data<Addr<Lobby>>, podcast_service: Data<Mutex<PodcastService>>) -> impl Responder {
        let id_num = from_str::<i32>(&id).unwrap();
        let mut podcast_service = podcast_service.lock().unwrap();
        let podcast = podcast_service.get_podcast_by_id(id_num);
        thread::spawn(move || {
            let mut podcast_service = PodcastService::new();
            podcast_service.refresh_podcast(podcast.clone(), lobby);
        });
        HttpResponse::Ok().json("Refreshing podcast")
}

#[put("/podcast/favored")]
pub async fn favorite_podcast(update_model: web::Json<PodcastFavorUpdateModel> ,
                              podcast_service_mutex: Data<Mutex<PodcastService>>) -> impl Responder {
    let mut podcast_service = podcast_service_mutex.lock().unwrap();
    podcast_service.update_favor_podcast( update_model.id, update_model.favored);
    HttpResponse::Ok().json("Favorited podcast")
}

#[get("/podcasts/favored")]
pub async fn get_favored_podcasts(podcast_service_mutex: Data<Mutex<PodcastService>>) -> impl Responder {
    let mut podcast_service = podcast_service_mutex.lock().unwrap();
    let podcasts = podcast_service.get_favored_podcasts();
    HttpResponse::Ok().json(podcasts)
}

#[put("/podcast/{id}/active")]
pub async fn update_active_podcast(id: web::Path<String>, podcast_service_mutex: Data<Mutex<PodcastService>>) -> impl Responder {
    let id_num = from_str::<i32>(&id).unwrap();
    let mut podcast_service = podcast_service_mutex.lock().unwrap();
    podcast_service.update_active_podcast(id_num);
    HttpResponse::Ok().json("Updated active podcast")
}