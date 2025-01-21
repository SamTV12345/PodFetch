use axum::{Extension, Json, Router};
use axum::extract::Path;
use axum::routing::{get, post};
use reqwest::StatusCode;
use crate::models::episode::Episode;
use crate::models::misc_models::{PodcastWatchedEpisodeModelWithPodcastEpisode, PodcastWatchedPostModel};
use crate::models::user::User;

use crate::utils::error::{CustomError, CustomErrorInner};

#[utoipa::path(
post,
path="/podcast/episode",
context_path="/api/v1",
responses(
(status = 200, description = "Logs a watchtime request.")),
tag="watchtime"
)]
pub async fn log_watchtime(
    podcast_watch: Json<PodcastWatchedPostModel>,
    requester: Extension<User>,
) -> Result<StatusCode, CustomError> {
    let podcast_episode_id = podcast_watch.0.podcast_episode_id.clone();
    Episode::log_watchtime(podcast_watch.0, requester.username.clone())?;
    log::debug!("Logged watchtime for episode: {}", podcast_episode_id);
    Ok(StatusCode::OK)
}

#[utoipa::path(
get,
path="/podcast/episode/lastwatched",
context_path="/api/v1",
responses(
(status = 200, description = "Gets the last watched podcast episodes.")),
tag="watchtime"
)]
pub async fn get_last_watched(Extension(requester): Extension<User>) ->
                                                                     Result<Json<Vec<PodcastWatchedEpisodeModelWithPodcastEpisode>>,
    CustomError> {

    let mut episodes = Episode::get_last_watched_episodes(&requester)?;
    episodes.sort_by(|a, b| a.date.cmp(&b.date).reverse());
    Ok(Json(episodes))
}

#[utoipa::path(
get,
path="/podcast/episode/{id}",
context_path="/api/v1",
responses(
(status = 200, description = "Gets watchtime by id.")),
tag="watchtime"
)]
pub async fn get_watchtime(
    Path(id): Path<String>,
    Extension(requester): Extension<User>,
) -> Result<Json<Episode>, CustomError> {
    let watchtime = Episode::get_watchtime(id, requester.username)?;
    match watchtime {
        None => Err(CustomErrorInner::NotFound.into()),
        Some(w) => Ok(Json(w)),
    }
}

pub fn get_watchtime_router() -> Router {
    Router::new()
        .route("/podcast/episode", post(log_watchtime))
        .route("/podcast/episode/lastwatched", get(get_last_watched))
        .route("/podcast/episode/{id}", get(get_watchtime))
}
