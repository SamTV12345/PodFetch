use crate::models::episode::{Episode, EpisodeDto};
use crate::models::misc_models::{
    PodcastWatchedEpisodeModelWithPodcastEpisode, PodcastWatchedPostModel,
};
use crate::models::user::User;
use axum::extract::Path;
use axum::{Extension, Json};
use reqwest::StatusCode;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::utils::error::{CustomError, CustomErrorInner};

#[utoipa::path(
post,
path="/podcasts/episode",
responses(
(status = 200, description = "Logs a watchtime request.")),
tag="watchtime"
)]
pub async fn log_watchtime(
    Extension(requester): Extension<User>,
    Json(podcast_watch): Json<PodcastWatchedPostModel>,
) -> Result<StatusCode, CustomError> {
    let podcast_episode_id = podcast_watch.podcast_episode_id.clone();
    Episode::log_watchtime(podcast_watch, requester.username.clone())?;
    log::debug!("Logged watchtime for episode: {podcast_episode_id}");
    Ok(StatusCode::OK)
}

#[utoipa::path(
get,
path="/podcasts/episode/lastwatched",
responses(
(status = 200, description = "Gets the last watched podcast episodes.", body= Vec<PodcastWatchedEpisodeModelWithPodcastEpisode>)),
tag="watchtime"
)]
pub async fn get_last_watched(
    Extension(requester): Extension<User>,
) -> Result<Json<Vec<PodcastWatchedEpisodeModelWithPodcastEpisode>>, CustomError> {
    let mut episodes = Episode::get_last_watched_episodes(&requester)?;
    episodes.sort_by(|a, b| a.date.cmp(&b.date).reverse());
    Ok(Json(episodes))
}

#[utoipa::path(
get,
path="/podcasts/episode/{id}",
responses(
(status = 200, description = "Gets watchtime by id.", body=EpisodeDto)),
tag="watchtime"
)]
pub async fn get_watchtime(
    Path(id): Path<String>,
    Extension(requester): Extension<User>,
) -> Result<Json<EpisodeDto>, CustomError> {
    let watchtime = Episode::get_watchtime(id, requester.username)?;
    match watchtime {
        None => Err(CustomErrorInner::NotFound.into()),
        Some(w) => Ok(Json(w.convert_to_episode_dto())),
    }
}

pub fn get_watchtime_router() -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(log_watchtime))
        .routes(routes!(get_last_watched))
        .routes(routes!(get_watchtime))
}
