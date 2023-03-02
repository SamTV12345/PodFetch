use std::borrow::ToOwned;
use reqwest::{ClientBuilder as AsyncClientBuilder};
use serde_json::{from_str, Value};
use crate::db::DB;
use crate::models::models::{PodCastAddModel, PodcastWatchedPostModel};
use crate::service::file_service::{create_podcast_directory_exists, download_podcast_image};
use crate::service::mapping_service::MappingService;
use crate::service::rust_service::{find_podcast as find_podcast_service, insert_podcast_episodes, schedule_episode_download};
use actix_web::{get, post, web, HttpResponse, Responder};
use actix_web::web::{Query};
use std::option::Option;
use std::thread;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OptionalId {
    last_podcast_episode: Option<String>,
}

impl OptionalId {
    pub fn new() -> Self {
        OptionalId { last_podcast_episode: None }
    }
}

#[get("/podcasts/{podcast}/search")]
pub async fn find_podcast(podcast: web::Path<String>) -> impl Responder {
    log::debug!("Searching for podcast: {}", podcast);
    let res = find_podcast_service(&podcast);
    HttpResponse::Ok().json(res.await)
}

#[get("/podcasts")]
pub async fn find_all_podcasts() -> impl Responder {
    let mut db = DB::new().unwrap();
    let mappingservice = MappingService::new();
    let podcasts = db.get_podcasts().unwrap();

    let mapped_podcasts = podcasts
        .into_iter()
        .map(|podcast| mappingservice.map_podcast_to_podcast_dto(&podcast)).collect::<Vec<_>>();
    HttpResponse::Ok().json(mapped_podcasts)
}


#[get("/podcast/{id}")]
pub async fn find_podcast_by_id(id: web::Path<String>) -> impl Responder {
    let id_num = from_str::<i32>(&id).unwrap();
    let mut db = DB::new().unwrap();
    let mappingservice = MappingService::new();
    let podcast = db.get_podcast(id_num).unwrap();
    let mapped_podcast = mappingservice.map_podcast_to_podcast_dto(&podcast);
    HttpResponse::Ok().json(mapped_podcast)
}

#[get("/podcast/{id}/episodes")]
pub async fn find_all_podcast_episodes_of_podcast(id: web::Path<String>, last_podcast_episode :
Query<OptionalId>)
    -> impl Responder {
    let last_podcast_episode = last_podcast_episode.into_inner();
    let id_num = from_str(&id).unwrap();
    let mut db = DB::new().unwrap();
    let mappingservice = MappingService::new();
    let res  = db.get_podcast_episodes_of_podcast(id_num,last_podcast_episode
        .last_podcast_episode ).unwrap();
    let mapped_podcasts = res
        .into_iter()
        .map(|podcast| mappingservice.map_podcastepisode_to_dto(&podcast)).collect::<Vec<_>>();
    HttpResponse::Ok().json(mapped_podcasts)
}




#[post("/podcast")]
pub async fn add_podcast(track_id: web::Json<PodCastAddModel>) -> impl Responder {
    let client = AsyncClientBuilder::new().build().unwrap();
    let res = client.get("https://itunes.apple.com/lookup?id=".to_owned()+&track_id
        .track_id
        .to_string())
        .send().await.unwrap();

    let mut db = DB::new().unwrap();

    let res  = res.json::<Value>().await.unwrap();


    db.add_podcast_to_database(unwrap_string(&res["results"][0]["collectionName"]),
                               unwrap_string(&res["results"][0]["collectionId"]),
                               unwrap_string(&res["results"][0]["feedUrl"]),
                               unwrap_string(&res["results"][0]["artworkUrl600"]));
    create_podcast_directory_exists(&unwrap_string(&res["results"][0]["collectionId"]));
    download_podcast_image(&unwrap_string(&res["results"][0]["collectionId"]),
                           &unwrap_string(&res["results"][0]["artworkUrl600"])).await;
    let podcast = db.get_podcast_episode_by_track_id(track_id.track_id).unwrap();

    match podcast {
        Some(podcast) => {
            thread::spawn(||{
                log::debug!("Inserting podcast episodes: {}", podcast.name);
                insert_podcast_episodes(podcast.clone());
                schedule_episode_download(podcast);
            });
        },
        None => {panic!("No podcast found")}
    }
    log::info!("Added podcast: {}", unwrap_string(&res["results"][0]["collectionName"]));
    HttpResponse::Ok()
}

#[post("/podcast/episode")]
pub async fn log_watchtime(podcast_watch: web::Json<PodcastWatchedPostModel>) -> impl Responder {
    let mut db = DB::new().unwrap();
    let podcast_episode_id = podcast_watch.0.podcast_episode_id.clone();
    db.log_watchtime(podcast_watch.0).expect("Error logging watchtime");
    log::debug!("Logged watchtime for episode: {}", podcast_episode_id);
    HttpResponse::Ok()
}


#[get("/podcast/episode/lastwatched")]
pub async fn get_last_watched() -> impl Responder {
    let mut db = DB::new().unwrap();
    let last_watched = db.get_last_watched_podcasts().unwrap();
    HttpResponse::Ok().json(last_watched)
}

#[get("/podcast/episode/{id}")]
pub async fn get_watchtime(id: web::Path<String>) -> impl Responder {
    let mut db = DB::new().unwrap();
    let watchtime = db.get_watchtime(&id).unwrap();
    HttpResponse::Ok().json(watchtime)
}



fn unwrap_string(value: &Value) -> String {
    return value.to_string().replace("\"", "");
}
