use actix_web::{post, get, web, HttpResponse, Responder};
use crate::db::DB;
use crate::models::models::PodcastWatchedPostModel;


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