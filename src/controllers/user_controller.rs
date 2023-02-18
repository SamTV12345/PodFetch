use std::future::Future;
use rocket::http::RawStr;
use rocket_contrib::json::Json;
use serde_json::Value;
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
    println!("podcast: {}", podcast);
    let res = find_podcast_service(podcast);
    Json(json!({
        "status": 200,
        "result": res,
    }))
}

#[post("/podcast", format = "application/json", data = "<track_id>")]
pub fn add_podcast(track_id: Json<PodCastAddModel>){
    println!("podcast: {}", track_id.track_id);
}
