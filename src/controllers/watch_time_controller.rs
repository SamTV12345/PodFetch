use crate::models::episode::Episode;
use crate::models::misc_models::PodcastWatchedPostModel;
use crate::models::user::User;

use crate::utils::error::{map_r2d2_error, CustomError};
use crate::DbPool;
use actix_web::web::Data;
use actix_web::{get, post, web, HttpResponse};

use std::ops::DerefMut;

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Logs a watchtime request.")),
tag="watchtime"
)]
#[post("/podcast/episode")]
pub async fn log_watchtime(
    podcast_watch: web::Json<PodcastWatchedPostModel>,
    conn: Data<DbPool>,
    requester: Option<web::ReqData<User>>,
) -> Result<HttpResponse, CustomError> {
    let podcast_episode_id = podcast_watch.0.podcast_episode_id.clone();
    Episode::log_watchtime(
        conn.get().map_err(map_r2d2_error)?.deref_mut(),
        podcast_watch.0,
        requester.unwrap().username.clone(),
    )?;
    log::debug!("Logged watchtime for episode: {}", podcast_episode_id);
    Ok(HttpResponse::Ok().body("Watchtime logged."))
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets the last watched podcast episodes.")),
tag="watchtime"
)]
#[get("/podcast/episode/lastwatched")]
pub async fn get_last_watched(
    conn: Data<DbPool>,
    requester: Option<web::ReqData<User>>,
) -> Result<HttpResponse, CustomError> {
    let designated_username = requester.unwrap().username.clone();

    let mut episodes = Episode::get_last_watched_episodes(
        designated_username,
        conn.get().map_err(map_r2d2_error)?.deref_mut(),
    )?;
    episodes.sort_by(|a, b| a.date.cmp(&b.date).reverse());
    Ok(HttpResponse::Ok().json(episodes))
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets watchtime by id.")),
tag="watchtime"
)]
#[get("/podcast/episode/{id}")]
pub async fn get_watchtime(
    id: web::Path<String>,
    conn: Data<DbPool>,
    requester: Option<web::ReqData<User>>,
) -> Result<HttpResponse, CustomError> {
    let designated_username = requester.unwrap().username.clone();
    let watchtime = Episode::get_watchtime(
        conn.get().map_err(map_r2d2_error)?.deref_mut(),
        id.into_inner(),
        designated_username,
    )?;
    Ok(HttpResponse::Ok().json(watchtime))
}
