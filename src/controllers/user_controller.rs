use std::future::Future;
use reqwest::blocking::{Client, ClientBuilder, Response};
use rocket::http::{RawStr, Status};
use rocket::http::ext::IntoCollection;
use rocket::response::status;
use rocket_contrib::json::Json;
use rocket_cors::{CorsOptions, Guard, Responder};
use rusqlite::Connection;
use serde_json::Value;
use crate::constants::constants::{DB_NAME};
use crate::db::DB;
use crate::models::itunes_models::Podcast;
use crate::models::models::{NewUser, PodCastAddModel, UserData};
use crate::service::file_service::{create_podcast_directory_exists, download_podcast_image};
use crate::service::mapping_service::MappingService;
use crate::service::rust_service::{find_podcast as find_podcast_service, insert_podcast_episodes, schedule_episode_download};

#[post("/users", format = "application/json")]
pub fn get_all() -> Json<Value> {
    Json(json!({
        "status": 200,
        "result": "test",
    }))
}

#[post("/newUser", format = "application/json", data = "<new_user>")]
pub fn new_user( new_user: Json<NewUser>) -> Json<Value> {
    Json(json!({
        "status": "test",
        "result": "test",
    }))
}

#[get("/getUser")]
pub fn find_user() -> Json<Value> {
    Json(json!({
        "status": 200,
        "result": "test",
    }))
}

#[get("/podcasts")]
pub fn find_all_podcasts() -> status::Accepted<Json<Value>> {
    let db = DB::new().unwrap();
    let mappingservice = MappingService::new();
    let podcasts = db.get_podcasts().unwrap();

    let mapped_podcasts = podcasts
        .into_iter()
        .map(|podcast| mappingservice.map_podcast_to_podcast_dto(&podcast)).collect::<Vec<_>>();
    status::Accepted(Some(Json(json!(mapped_podcasts))))
}


#[get("/podcast/<id>")]
pub fn find_podcast_by_id(id: i64) -> status::Accepted<Json<Value>> {
    let db = DB::new().unwrap();
    let mappingservice = MappingService::new();
    let podcast = db.get_podcast(id).unwrap();
    let mapped_podcast = mappingservice.map_podcast_to_podcast_dto(&podcast);
    status::Accepted(Some(Json(json!(mapped_podcast))))
}

#[get("/podcast/<id>/episodes")]
pub fn find_all_podcast_episodes_of_podcast(id: i64) -> status::Accepted<Json<Value>> {
    let db = DB::new().unwrap();
    let mappingservice = MappingService::new();

    let res  = db.get_podcast_episodes_of_podcast(id).unwrap();
    let mapped_podcasts = res
        .into_iter()
        .map(|podcast| mappingservice.map_podcastepisode_to_dto(&podcast)).collect::<Vec<_>>();
    status::Accepted(Some(Json(json!(mapped_podcasts))))
}


#[get("/podcast?<podcast>")]
pub fn find_podcast(podcast: &RawStr) ->Json<Value> {
    let res = find_podcast_service(podcast);
    Json(json!({
        "code": 200,
        "result": res,
    }))
}

fn cors_options_all() -> CorsOptions {
    // You can also deserialize this
    Default::default()
}

/// Manually mount an OPTIONS route for your own handling
#[options("/podcast")]
pub fn manual_options(cors: Guard<'_>) -> Responder<&str> {
    cors.responder("Manual OPTIONS preflight handling")
}

#[post("/podcast", format = "application/json", data = "<track_id>")]
pub fn add_podcast(track_id: Json<PodCastAddModel>){
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
}



fn unwrap_string(value: &Value) -> String {
    return value.to_string().replace("\"", "");
}
