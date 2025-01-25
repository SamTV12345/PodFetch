use axum::{Extension, Json};
use axum::extract::{Path, Query};
use crate::models::episode::{Episode, EpisodeDto};
use crate::models::session::Session;
use crate::utils::error::{CustomError, CustomErrorInner};
use crate::utils::time::get_current_timestamp;
use chrono::DateTime;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[derive(Serialize, Deserialize)]
pub struct EpisodeActionResponse {
    actions: Vec<Episode>,
    timestamp: i64,
}

#[derive(Serialize, Deserialize)]
pub struct EpisodeActionPostResponse {
    update_urls: Vec<String>,
    timestamp: i64,
}

#[derive(Serialize, Deserialize)]
pub struct EpisodeSinceRequest {
    since: i64,
    podcast: Option<String>,
    device: Option<String>,
    aggregate: Option<String>,
}

#[utoipa::path(
    get,
    path="/episodes/{username}",
    responses(
        (status = 200, description = "Gets the episode actions of a user."),
        (status = 403, description = "Forbidden"),
    ),
    tag="gpodder"
)]
pub async fn get_episode_actions(
    Extension(opt_flag): Extension<Option<Session>>,
    Query(since): Query<EpisodeSinceRequest>,
    Path(username): Path<String>,
) -> Result<Json<EpisodeActionResponse>, CustomError> {

    match opt_flag {
        Some(flag) => {
            let username = username.clone();
            if flag.username != username.clone() {
                return Err(CustomErrorInner::Forbidden.into());
            }

            let since_date = DateTime::from_timestamp(since.since, 0).map(|v| v.naive_utc());
            let mut actions = Episode::get_actions_by_username(
                username.clone(),
                since_date,
                since.device.clone(),
                since.aggregate.clone(),
                since.podcast.clone(),
            )
            .await;

            if let Some(device) = since.device.clone() {
                actions.iter_mut().for_each(|a| {
                    a.device = device.clone();
                });
            }

            Ok(Json(EpisodeActionResponse {
                actions,
                timestamp: get_current_timestamp(),
            }))
        }
        None => Err(CustomErrorInner::Forbidden.into()),
    }
}

#[
    utoipa::path(
        post,
        path="/episodes/{username}",
        responses(
            (status = 200, description = "Uploads episode actions."),
            (status = 403, description = "Forbidden"),
        ),
        tag="gpodder"
    )
]
pub async fn upload_episode_actions(
    username: Path<String>,
    Extension(opt_flag): Extension<Option<Session>>,
    Json(podcast_episode): Json<Vec<EpisodeDto>>,
) -> Result<Json<EpisodeActionPostResponse>, CustomError> {
    match opt_flag {
        Some(flag) => {
            if flag.username != username.clone() {
                return Err(CustomErrorInner::Forbidden.into());
            }
            let mut inserted_episodes: Vec<Episode> = vec![];
            podcast_episode.iter().for_each(|episode| {
                let episode = Episode::convert_to_episode(episode, username.clone());
                inserted_episodes.push(Episode::insert_episode(&episode.clone()).unwrap());
            });
            Ok(Json(EpisodeActionPostResponse {
                update_urls: vec![],
                timestamp: get_current_timestamp(),
            }))
        }
        None => Err(CustomErrorInner::Forbidden.into()),
    }
}

pub fn get_gpodder_episodes_router() -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(get_episode_actions))
        .routes(routes!(upload_episode_actions))
}
