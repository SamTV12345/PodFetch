use crate::db::DB;
use crate::models::models::PodcastWatchedPostModel;
use actix_web::web::Data;
use actix_web::{get, post, web, HttpResponse, Responder};
use std::sync::{Mutex};
use crate::DbPool;
use crate::mutex::LockResultExt;

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Logs a watchtime request.")),
tag="watchtime"
)]
#[post("/podcast/episode")]
pub async fn log_watchtime(podcast_watch: web::Json<PodcastWatchedPostModel>, conn: Data<DbPool>) ->
                                                                                             impl
Responder {
    let podcast_episode_id = podcast_watch.0.podcast_episode_id.clone();
    DB::log_watchtime(&mut conn.get().unwrap(),podcast_watch.0)
        .expect("Error logging watchtime");
    log::debug!("Logged watchtime for episode: {}", podcast_episode_id);
    HttpResponse::Ok()
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets the last watched podcasts.")),
tag="watchtime"
)]
#[get("/podcast/episode/lastwatched")]
pub async fn get_last_watched(db: Data<Mutex<DB>>, conn: Data<DbPool>) -> impl Responder {
    let mut db = db.lock().ignore_poison();
    let last_watched = db.get_last_watched_podcasts(&mut conn.get().unwrap()).unwrap();
    HttpResponse::Ok().json(last_watched)
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets watchtime by id.")),
tag="watchtime"
)]
#[get("/podcast/episode/{id}")]
pub async fn get_watchtime(id: web::Path<String>, conn: Data<DbPool>) -> impl Responder {
    let watchtime = DB::get_watchtime(&mut conn.get().unwrap(),&id).unwrap();
    HttpResponse::Ok().json(watchtime)
}
