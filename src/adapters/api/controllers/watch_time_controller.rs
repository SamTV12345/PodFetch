

use crate::utils::error::CustomError;
use actix_web::{get, post, web, HttpResponse};
use crate::adapters::api::models::podcast::podcast_watched_post_model::PodcastWatchedPostModel;
use crate::adapters::api::models::user::user::UserDto;
use crate::application::services::episode::episode_service::EpisodeService;

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Logs a watchtime request.")),
tag="watchtime"
)]
#[post("/podcast/episode")]
pub async fn log_watchtime(
    podcast_watch: web::Json<PodcastWatchedPostModel>,
    requester: Option<web::ReqData<UserDto>>,
) -> Result<HttpResponse, CustomError> {
    let podcast_episode_id = podcast_watch.into_inner();
    EpisodeService::log_watchtime(
        &podcast_episode_id.podcast_episode_id,
        podcast_episode_id.time,
        &*requester.unwrap().username.clone(),
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
    requester: Option<web::ReqData<UserDto>>,
) -> Result<HttpResponse, CustomError> {
    let designated_username = requester.unwrap().username.clone();

    let mut episodes = EpisodeService::get_last_watched_episodes(
        designated_username,
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
    requester: Option<web::ReqData<User>>,
) -> Result<HttpResponse, CustomError> {
    let designated_username = requester.unwrap().username.clone();
    let watchtime = Episode::get_watchtime(
        id.into_inner(),
        designated_username,
    )?;
    Ok(HttpResponse::Ok().json(watchtime))
}
