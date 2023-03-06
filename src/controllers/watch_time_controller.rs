use actix_web::{post, get, web, HttpResponse, Responder};
use crate::db::DB;
use crate::models::models::PodcastWatchedPostModel;

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Logs a watchtime request.")),
tag="watchtime"
)]
#[post("/podcast/episode")]
pub async fn log_watchtime(podcast_watch: web::Json<PodcastWatchedPostModel>) -> impl Responder {
    let mut db = DB::new().unwrap();
    let podcast_episode_id = podcast_watch.0.podcast_episode_id.clone();
    db.log_watchtime(podcast_watch.0).expect("Error logging watchtime");
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
pub async fn get_last_watched() -> impl Responder {
    let mut db = DB::new().unwrap();
    let last_watched = db.get_last_watched_podcasts().unwrap();
    HttpResponse::Ok().json(last_watched)
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets watchtime by id.")),
tag="watchtime"
)]
#[get("/podcast/episode/{id}")]
pub async fn get_watchtime(id: web::Path<String>) -> impl Responder {
    let mut db = DB::new().unwrap();
    let watchtime = db.get_watchtime(&id).unwrap();
    HttpResponse::Ok().json(watchtime)
}