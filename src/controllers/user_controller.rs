use std::future::Future;
use reqwest::blocking::{Client, ClientBuilder, Response};
use rocket::http::RawStr;
use rocket_contrib::json::Json;
use rusqlite::Connection;
use serde_json::Value;
use crate::constants::constants::DB_NAME;
use crate::db::DB;
use crate::models::models::{NewUser, PodCastAddModel, UserData};
use crate::service::rust_service::find_podcast as find_podcast_service;

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

#[get("/podcast?<podcast>")]
pub fn find_podcast(podcast: &RawStr) ->Json<Value> {
    let res = find_podcast_service(podcast);
    Json(json!({
        "status": 200,
        "result": res,
    }))
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
                               unwrap_string(&res["results"][0]["feedUrl"]));
}



fn unwrap_string(value: &Value) -> String {
    return value.to_string().replace("\"", "");
}
