use crate::app_state::AppState;
use crate::gpodder::{
    EpisodeActionPostResponse, EpisodeActionResponse, EpisodeSinceRequest, ensure_session_user,
    map_gpodder_error, parse_since_epoch,
};
use crate::history::{EpisodeDto, map_episode_dto_to_episode, map_episode_to_dto};
use crate::usecases::watchtime::WatchtimeUseCase as WatchtimeService;
use axum::extract::{Path, Query, State};
use axum::{Extension, Json};
use common_infrastructure::error::CustomError;
use common_infrastructure::path::trim_from_path;
use common_infrastructure::time::get_current_timestamp;
use podfetch_domain::session::Session;
use serde::Serialize;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

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
        inserted_episodes.push(WatchtimeService::upsert_episode_by_guid(episode).unwrap());
    });
    Ok(Json(EpisodeActionPostResponse {
        update_urls: vec![],
        timestamp: get_current_timestamp(),
    }))
}

#[derive(Serialize)]
pub struct GpodderFavoriteEpisode {
    pub title: String,
    pub url: String,
    pub podcast_title: String,
    pub podcast_url: String,
}

#[utoipa::path(
    get,
    path="/favorites/{username}",
    responses(
        (status = 200, description = "Gets the user's favorite episodes."),
        (status = 403, description = "Forbidden"),
    ),
    tag="gpodder"
)]
pub async fn get_favorites(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Extension(flag): Extension<Session>,
) -> Result<Json<Vec<GpodderFavoriteEpisode>>, CustomError> {
    use crate::services::podcast::service::PodcastService;
    use crate::usecases::podcast_episode::PodcastEpisodeUseCase;

    let username = trim_from_path(&username);
    ensure_session_user::<CustomError>(&flag.username, username.0).map_err(map_gpodder_error)?;

    let favorites = state
        .favorite_podcast_episode_service
        .get_favorites_by_user_id(flag.user_id)?;

    let mut result = Vec::new();
    for fav in favorites {
        if let Ok(Some(episode)) =
            PodcastEpisodeUseCase::get_podcast_episode_by_internal_id(fav.episode_id)
        {
            let podcast = PodcastService::get_podcast_by_id(episode.podcast_id);

            result.push(GpodderFavoriteEpisode {
                title: episode.name,
                url: episode.url,
                podcast_title: podcast.name.clone(),
                podcast_url: podcast.rssfeed.clone(),
            });
        }
    }

    Ok(Json(result))
}

pub fn get_gpodder_episodes_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_episode_actions))
        .routes(routes!(upload_episode_actions))
        .routes(routes!(get_favorites))
}

#[cfg(test)]
pub mod tests {
    use crate::app_state::AppState;
    use crate::gpodder_api::auth::test_support::tests::create_auth_gpodder;
    use crate::gpodder_api::episodes::gpodder_episodes::EpisodeActionResponse;
    use crate::test_support::tests::handle_test_startup;
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

    #[serial]
    #[tokio::test]
    async fn test_guid_fallback_updates_existing_episode_action() {
        let mut test_server = handle_test_startup().await;
        let state = app_state();
        let user = state
            .user_admin_service
            .create_user(UserTestDataBuilder::new().build())
            .unwrap();
        create_auth_gpodder(&mut test_server, &user).await;

        let guid = uuid::Uuid::new_v4().to_string();
        let timestamp = "2026-01-01T12:00:00";

        // Upload episode action with original URL
        let resp = test_server
            .test_server
            .post(&format!("/api/2/episodes/{}", user.username))
            .json(&serde_json::json!([{
                "podcast": "https://example.com/feed.xml",
                "episode": "https://example.com/episode1.mp3",
                "guid": guid,
                "action": "play",
                "device": "device-a",
                "timestamp": timestamp,
                "started": 0,
                "position": 60,
                "total": 300
            }]))
            .await;
        assert_eq!(resp.status_code(), 200);

        // Upload same episode with a local URL but same GUID — should update, not duplicate
        let resp = test_server
            .test_server
            .post(&format!("/api/2/episodes/{}", user.username))
            .json(&serde_json::json!([{
                "podcast": "http://192.168.1.50:8000/rss/1",
                "episode": "http://192.168.1.50:8000/podcasts/episode1.mp3",
                "guid": guid,
                "action": "play",
                "device": "device-b",
                "timestamp": "2026-01-01T12:05:00",
                "started": 60,
                "position": 180,
                "total": 300
            }]))
            .await;
        assert_eq!(resp.status_code(), 200);

        // Fetch all actions — should have updated position, not two separate entries
        let resp = test_server
            .test_server
            .get(&format!("/api/2/episodes/{}?since=0", user.username))
            .await;
        assert_eq!(resp.status_code(), 200);
        let json = resp.json::<EpisodeActionResponse>();

        let matching: Vec<_> = json
            .actions
            .iter()
            .filter(|a| a.guid.as_deref() == Some(&guid))
            .collect();

        assert_eq!(
            matching.len(),
            1,
            "GUID fallback should update existing entry, not create a duplicate"
        );
        assert_eq!(
            matching[0].position,
            Some(180),
            "Position should be updated to 180"
        );
    }

    #[serial]
    #[tokio::test]
    async fn test_get_favorites_empty() {
        let mut test_server = handle_test_startup().await;
        let state = app_state();
        let user = state
            .user_admin_service
            .create_user(UserTestDataBuilder::new().build())
            .unwrap();
        create_auth_gpodder(&mut test_server, &user).await;

        let resp = test_server
            .test_server
            .get(&format!("/api/2/favorites/{}.json", user.username))
            .await;
        assert_eq!(resp.status_code(), 200);
        let json = resp.json::<Vec<serde_json::Value>>();
        assert!(json.is_empty());
    }
}
