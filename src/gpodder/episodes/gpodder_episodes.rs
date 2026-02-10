use crate::models::episode::{Episode, EpisodeDto};
use crate::models::session::Session;
use crate::utils::error::ErrorSeverity::Warning;
use crate::utils::error::{CustomError, CustomErrorInner};
use crate::utils::gpodder_trimmer::trim_from_path;
use crate::utils::time::get_current_timestamp;
use axum::extract::{Path, Query};
use axum::{Extension, Json};
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
    Extension(flag): Extension<Session>,
    Query(since): Query<EpisodeSinceRequest>,
    Path(username): Path<String>,
) -> Result<Json<EpisodeActionResponse>, CustomError> {
    let username = trim_from_path(&username);
    if flag.username != username.0 {
        return Err(CustomErrorInner::Forbidden(Warning).into());
    }

    let since_date = DateTime::from_timestamp(since.since, 0).map(|v| v.naive_utc());
    let mut actions = Episode::get_actions_by_username(
        username.0,
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
    Path(username): Path<String>,
    Extension(flag): Extension<Session>,
    Json(podcast_episode): Json<Vec<EpisodeDto>>,
) -> Result<Json<EpisodeActionPostResponse>, CustomError> {
    let username = trim_from_path(&username);
    if flag.username != username.0 {
        return Err(CustomErrorInner::Forbidden(Warning).into());
    }
    let mut inserted_episodes: Vec<Episode> = vec![];
    podcast_episode.iter().for_each(|episode| {
        let episode = Episode::convert_to_episode(episode, username.0.to_string());
        inserted_episodes.push(Episode::insert_episode(&episode.clone()).unwrap());
    });
    Ok(Json(EpisodeActionPostResponse {
        update_urls: vec![],
        timestamp: get_current_timestamp(),
    }))
}

pub fn get_gpodder_episodes_router() -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(get_episode_actions))
        .routes(routes!(upload_episode_actions))
}

#[cfg(test)]
pub mod tests {
    use crate::commands::startup::tests::handle_test_startup;
    use crate::gpodder::episodes::gpodder_episodes::EpisodeActionResponse;
    use crate::utils::auth::tests::create_auth_gpodder;
    use crate::utils::test_builder::user_test_builder::tests::UserTestDataBuilder;
    use serial_test::serial;

    #[serial]
    #[tokio::test]
    async fn test_get_episodes_action() {
        let mut test_server = handle_test_startup().await;
        let mut user = UserTestDataBuilder::new().build();
        user.insert_user().unwrap();
        create_auth_gpodder(&mut test_server, &user).await;
        let resp = test_server
            .test_server
            .get(&format!("/api/2/episodes/{}?since=0", user.username))
            .await;
        assert_eq!(resp.status_code(), 200);
        let json = resp.json::<EpisodeActionResponse>();
        assert_eq!(json.actions.len(), 0);
    }
}
