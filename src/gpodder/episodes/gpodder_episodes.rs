use crate::adapters::api::mappers::episode::{map_episode_dto_to_episode, map_episode_to_dto};
use crate::application::usecases::watchtime::WatchtimeUseCase as WatchtimeService;
use common_infrastructure::error::ErrorSeverity::Warning;
use common_infrastructure::error::{CustomError, CustomErrorInner};
use common_infrastructure::path::trim_from_path;
use common_infrastructure::time::get_current_timestamp;
use axum::extract::{Path, Query};
use axum::{Extension, Json};
use podfetch_domain::session::Session;
use podfetch_web::gpodder::{
    EpisodeActionPostResponse, EpisodeActionResponse, EpisodeSinceRequest, GpodderControllerError,
    ensure_session_user, parse_since_epoch,
};
use podfetch_web::history::EpisodeDto;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

fn map_gpodder_error(error: GpodderControllerError<CustomError>) -> CustomError {
    match error {
        GpodderControllerError::Forbidden => CustomErrorInner::Forbidden(Warning).into(),
        GpodderControllerError::BadRequest(message) => {
            CustomErrorInner::BadRequest(message, Warning).into()
        }
        GpodderControllerError::Service(error) => error,
    }
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
    ensure_session_user::<CustomError>(&flag.username, username.0).map_err(map_gpodder_error)?;
    let since_date = parse_since_epoch::<CustomError>(since.since).map_err(map_gpodder_error)?;
    let mut actions = WatchtimeService::get_actions_by_username(
        username.0,
        since_date,
        since.device.clone(),
        since.aggregate.clone(),
        since.podcast.clone(),
    )?;

    if let Some(device) = since.device.clone() {
        actions.iter_mut().for_each(|a| {
            a.device = device.clone();
        });
    }

    Ok(Json(EpisodeActionResponse {
        actions: actions
            .iter()
            .map(|episode| map_episode_to_dto(&episode.clone().into()))
            .collect(),
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
    ensure_session_user::<CustomError>(&flag.username, username.0).map_err(map_gpodder_error)?;
    let mut inserted_episodes = vec![];
    podcast_episode.iter().for_each(|episode| {
        let episode = map_episode_dto_to_episode(episode, username.0.to_string());
        inserted_episodes.push(WatchtimeService::insert_episode(episode).unwrap());
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
    use crate::app_state::AppState;
    use crate::commands::startup::tests::handle_test_startup;
    use crate::gpodder::episodes::gpodder_episodes::EpisodeActionResponse;
    use crate::gpodder::auth::test_support::tests::create_auth_gpodder;
    use crate::test_utils::test_builder::user_test_builder::tests::UserTestDataBuilder;
    use serial_test::serial;

    fn app_state() -> AppState {
        AppState::new()
    }

    #[serial]
    #[tokio::test]
    async fn test_get_episodes_action() {
        let mut test_server = handle_test_startup().await;
        let state = app_state();
        let user = state
            .user_admin_service
            .create_user(UserTestDataBuilder::new().build())
            .unwrap();
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


