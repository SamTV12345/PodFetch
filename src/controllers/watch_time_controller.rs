use crate::models::episode::{Episode, EpisodeDto};
use crate::models::misc_models::{
    PodcastWatchedEpisodeModelWithPodcastEpisode, PodcastWatchedPostModel,
};
use crate::models::user::User;
use axum::extract::Path;
use axum::http::HeaderMap;
use axum::{Extension, Json};
use reqwest::StatusCode;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::utils::error::ErrorSeverity::Debug;
use crate::utils::error::{CustomError, CustomErrorInner};
use crate::utils::url_builder::{resolve_server_url_from_headers, rewrite_env_server_url_prefix};

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
    headers: HeaderMap,
) -> Result<Json<Vec<PodcastWatchedEpisodeModelWithPodcastEpisode>>, CustomError> {
    let server_url = resolve_server_url_from_headers(&headers);
    let episodes = Episode::get_last_watched_episodes(&requester)?
        .into_iter()
        .map(|mut item| {
            item.podcast_episode.local_url =
                rewrite_env_server_url_prefix(&item.podcast_episode.local_url, &server_url);
            item.podcast_episode.local_image_url =
                rewrite_env_server_url_prefix(&item.podcast_episode.local_image_url, &server_url);
            item.podcast.image_url =
                rewrite_env_server_url_prefix(&item.podcast.image_url, &server_url);
            item.podcast.podfetch_feed =
                rewrite_env_server_url_prefix(&item.podcast.podfetch_feed, &server_url);
            item
        })
        .collect();
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
    let watchtime = Episode::get_watchtime(&id, &requester.username)?;
    match watchtime {
        None => Err(CustomErrorInner::NotFound(Debug).into()),
        Some(w) => Ok(Json(w.convert_to_episode_dto())),
    }
}

pub fn get_watchtime_router() -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(log_watchtime))
        .routes(routes!(get_last_watched))
        .routes(routes!(get_watchtime))
}
