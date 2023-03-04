use std::thread;
use actix_web::{HttpResponse, Responder, web};
use serde_json::{from_str, Value};
use crate::db::DB;
use crate::service::mapping_service::MappingService;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use actix_web::{get, post};
use crate::models::models::PodCastAddModel;
use crate::service::file_service::FileService;
use crate::service::rust_service::schedule_episode_download;
use crate::unwrap_string;
use crate::service::rust_service::{find_podcast as find_podcast_service};
use reqwest::{ClientBuilder as AsyncClientBuilder};


#[get("/podcast/{id}")]
    pub async fn find_podcast_by_id( id: web::Path<String>) -> impl Responder {
        let id_num = from_str::<i32>(&id).unwrap();
        let mut db = DB::new().unwrap();
        let podcast = db.get_podcast(id_num).unwrap();
        let mapping_service = MappingService::new();
        let mapped_podcast = mapping_service.map_podcast_to_podcast_dto(&podcast);
        HttpResponse::Ok().json(mapped_podcast)
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

#[get("/podcasts/{podcast}/search")]
pub async fn find_podcast(podcast: web::Path<String>) -> impl Responder {
    log::debug!("Searching for podcast: {}", podcast);
    let res = find_podcast_service(&podcast);
    HttpResponse::Ok().json(res.await)
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
    FileService::create_podcast_directory_exists(&unwrap_string
        (&res["results"][0]["collectionId"]));
    FileService::download_podcast_image(&unwrap_string(&res["results"][0]["collectionId"]),
                                        &unwrap_string(&res["results"][0]["artworkUrl600"])).await;
    let podcast = db.get_podcast_episode_by_track_id(track_id.track_id).unwrap();

    match podcast {
        Some(podcast) => {
            thread::spawn(||{
                log::debug!("Inserting podcast episodes: {}", podcast.name);
                PodcastEpisodeService::insert_podcast_episodes(podcast.clone());
                schedule_episode_download(podcast);
            });
        },
        None => {panic!("No podcast found")}
    }
    log::info!("Added podcast: {}", unwrap_string(&res["results"][0]["collectionName"]));
    HttpResponse::Ok()
}