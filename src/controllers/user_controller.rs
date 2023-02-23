use std::future::Future;
use actix::ActorStreamExt;
use reqwest::blocking::{Client, ClientBuilder, Response};
use rusqlite::Connection;
use serde_json::{from_str, Value};
use crate::constants::constants::{DB_NAME};
use crate::db::DB;
use crate::models::itunes_models::Podcast;
use crate::models::models::{NewUser, PodCastAddModel, UserData};
use crate::service::file_service::{create_podcast_directory_exists, download_podcast_image};
use crate::service::mapping_service::MappingService;
use crate::service::rust_service::{find_podcast as find_podcast_service, insert_podcast_episodes, schedule_episode_download};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};


#[get("/podcasts")]
pub async fn find_all_podcasts() -> impl Responder {
    let db = DB::new().unwrap();
    let mappingservice = MappingService::new();
    let podcasts = db.get_podcasts().unwrap();

    let mapped_podcasts = podcasts
        .into_iter()
        .map(|podcast| mappingservice.map_podcast_to_podcast_dto(&podcast)).collect::<Vec<_>>();
    HttpResponse::Ok().json(mapped_podcasts)
}


#[get("/podcast/{id}")]
pub async fn find_podcast_by_id(id: web::Path<String>) -> impl Responder {
    println!("id: {}", id);
    let id_num = from_str::<i64>(&id).unwrap();
    let db = DB::new().unwrap();
    let mappingservice = MappingService::new();
    let podcast = db.get_podcast(id_num).unwrap();
    let mapped_podcast = mappingservice.map_podcast_to_podcast_dto(&podcast);
    HttpResponse::Ok().json(mapped_podcast)
}

#[get("/podcast/{id}/episodes")]
pub async fn find_all_podcast_episodes_of_podcast(id: web::Path<String>) -> impl Responder {
    let id_num = from_str(&id).unwrap();
    let db = DB::new().unwrap();
    let mappingservice = MappingService::new();

    let res  = db.get_podcast_episodes_of_podcast(id_num).unwrap();
    let mapped_podcasts = res
        .into_iter()
        .map(|podcast| mappingservice.map_podcastepisode_to_dto(&podcast)).collect::<Vec<_>>();
    HttpResponse::Ok().json(mapped_podcasts)
}


#[get("/podcast?<podcast>")]
pub async fn find_podcast(podcast: String) -> impl Responder {
    let res = find_podcast_service(&podcast);
    HttpResponse::Ok().json(res)
}

#[post("/podcast")]
pub async fn add_podcast(track_id: web::Json<PodCastAddModel>) -> impl Responder {
    println!("Podcast: {}", track_id.track_id);
    let client = ClientBuilder::new().build().unwrap();
    let mut res = client.get("https://itunes.apple.com/lookup?id=".to_owned()+&track_id
        .track_id
        .to_string())
        .send().unwrap();

    let db = DB::new().unwrap();

    let res  = res.json::<Value>().unwrap();


    db.add_podcast_to_database(unwrap_string(&res["results"][0]["collectionName"]),
                               unwrap_string(&res["results"][0]["collectionId"]),
                               unwrap_string(&res["results"][0]["feedUrl"]),
                               unwrap_string(&res["results"][0]["artworkUrl600"]));
    create_podcast_directory_exists(&unwrap_string(&res["results"][0]["collectionId"]));
    download_podcast_image(&unwrap_string(&res["results"][0]["collectionId"]),
                           &unwrap_string(&res["results"][0]["artworkUrl600"]));
    let podcast = db.get_podcast_episode_by_track_id(track_id.track_id).unwrap();
    match  { podcast}{
        Some(podcast) => {
            let podcast_cloned = podcast.clone();
            insert_podcast_episodes(podcast);
            schedule_episode_download(podcast_cloned)
        },
        None => {println!("No podcast found")}
    }
    HttpResponse::Ok()
}



fn unwrap_string(value: &Value) -> String {
    return value.to_string().replace("\"", "");
}
