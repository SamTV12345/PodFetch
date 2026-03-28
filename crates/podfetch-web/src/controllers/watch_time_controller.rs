use crate::podcast_episode_dto::PodcastEpisodeDto;
use crate::app_state::AppState;
use axum::extract::Path;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::{Extension, Json};
use podfetch_domain::user::User;
use crate::history::EpisodeDto;
use crate::podcast::PodcastDto;
use crate::watchtime::{
    self, PodcastWatchedEpisodeModelWithPodcastEpisode, PodcastWatchedPostModel,
    WatchtimeControllerError,
};
use reqwest::StatusCode;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use common_infrastructure::error::ErrorSeverity::Debug;
use common_infrastructure::error::{CustomError, CustomErrorInner};
use crate::url_rewriting::create_url_rewriter;

pub type LastWatchedItem =
    PodcastWatchedEpisodeModelWithPodcastEpisode<PodcastEpisodeDto, PodcastDto, EpisodeDto>;

#[utoipa::path(
post,
path="/podcasts/episode",
responses(
(status = 200, description = "Logs a watchtime request.")),
tag="watchtime"
)]
pub async fn log_watchtime(
    State(state): State<AppState>,
    Extension(requester): Extension<User>,
    Json(podcast_watch): Json<PodcastWatchedPostModel>,
) -> Result<StatusCode, CustomError> {
    let podcast_episode_id = podcast_watch.podcast_episode_id.clone();
    watchtime::log_watchtime(
        state.watchtime_service.as_ref(),
        requester.username.clone(),
        podcast_watch,
    )
    .map_err(map_watchtime_error)?;
    log::debug!("Logged watchtime for episode: {podcast_episode_id}");
    Ok(StatusCode::OK)
}

#[utoipa::path(
get,
path="/podcasts/episode/lastwatched",
responses(
(status = 200, description = "Gets the last watched podcast episodes.", body= Vec<LastWatchedItem>)),
tag="watchtime"
)]
pub async fn get_last_watched(
    State(state): State<AppState>,
    Extension(requester): Extension<User>,
    headers: HeaderMap,
) -> Result<Json<Vec<LastWatchedItem>>, CustomError> {
    let rewriter = create_url_rewriter(&headers);
    let episodes =
        watchtime::get_last_watched(state.watchtime_service.as_ref(), &requester.username)
            .map_err(map_watchtime_error)?
            .into_iter()
            .map(|mut item| {
                rewriter.rewrite_in_place(&mut item.podcast_episode.local_url);
                rewriter.rewrite_in_place(&mut item.podcast_episode.local_image_url);
                item.podcast.rewrite_urls(&rewriter);
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
    State(state): State<AppState>,
    Path(id): Path<String>,
    Extension(requester): Extension<User>,
) -> Result<Json<EpisodeDto>, CustomError> {
    watchtime::get_watchtime(state.watchtime_service.as_ref(), &id, &requester.username)
        .map(Json)
        .map_err(map_watchtime_error)
}

fn map_watchtime_error(error: WatchtimeControllerError<CustomError>) -> CustomError {
    match error {
        WatchtimeControllerError::NotFound => CustomErrorInner::NotFound(Debug).into(),
        WatchtimeControllerError::Service(error) => error,
    }
}

pub fn get_watchtime_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(log_watchtime))
        .routes(routes!(get_last_watched))
        .routes(routes!(get_watchtime))
}

#[cfg(test)]
mod tests {
    use podfetch_persistence::db::get_connection;
    use podfetch_persistence::schema::podcast_episodes::dsl as pe_dsl;
    use crate::test_support::tests::handle_test_startup;
    use podfetch_persistence::podcast_episode::PodcastEpisodeEntity as PodcastEpisode;
    use serde_json::json;
    use serial_test::serial;
    use uuid::Uuid;
    use crate::history::{EpisodeAction, EpisodeDto};

    fn insert_episode(
        podcast_id: i32,
        episode_id: &str,
        guid: &str,
        title: &str,
    ) -> PodcastEpisode {
        use diesel::ExpressionMethods;
        use diesel::RunQueryDsl;

        diesel::insert_into(pe_dsl::podcast_episodes)
            .values((
                pe_dsl::podcast_id.eq(podcast_id),
                pe_dsl::episode_id.eq(episode_id.to_string()),
                pe_dsl::name.eq(title.to_string()),
                pe_dsl::url.eq(format!("https://example.com/{episode_id}.mp3")),
                pe_dsl::date_of_recording.eq("2026-03-01T00:00:00Z".to_string()),
                pe_dsl::image_url.eq("http://localhost:8080/ui/default.jpg".to_string()),
                pe_dsl::total_time.eq(1800),
                pe_dsl::description.eq("watchtime test".to_string()),
                pe_dsl::guid.eq(guid.to_string()),
                pe_dsl::deleted.eq(false),
                pe_dsl::episode_numbering_processed.eq(false),
            ))
            .get_result::<PodcastEpisode>(&mut get_connection())
            .unwrap()
    }

    fn assert_client_error_status(status: u16) {
        assert!(
            (400..500).contains(&status),
            "expected 4xx status, got {status}"
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_log_watchtime_and_fetch_by_episode_id() {
        let server = handle_test_startup().await;

        let podcast = crate::services::podcast::service::PodcastService::add_podcast_to_database(
            "Watchtime Podcast",
            "watchtime-podcast",
            "https://example.com/watchtime.xml",
            "http://localhost:8080/ui/default.jpg",
            "watchtime-podcast",
        )
        .unwrap();
        let podcast_episode = insert_episode(
            podcast.id,
            "watchtime-episode-1",
            "watchtime-guid-1",
            "Watchtime Episode 1",
        );

        let log_response = server
            .test_server
            .post("/api/v1/podcasts/episode")
            .json(&json!({
                "podcastEpisodeId": podcast_episode.episode_id,
                "time": 133
            }))
            .await;
        assert_eq!(log_response.status_code(), 200);

        let get_response = server
            .test_server
            .get("/api/v1/podcasts/episode/watchtime-episode-1")
            .await;
        assert_eq!(get_response.status_code(), 200);
        let episode = get_response.json::<EpisodeDto>();
        assert_eq!(episode.position, Some(133));
        assert_eq!(episode.started, Some(133));
        assert_eq!(episode.action, EpisodeAction::Play);

        let last_watched = server
            .test_server
            .get("/api/v1/podcasts/episode/lastwatched")
            .await;
        assert_eq!(last_watched.status_code(), 200);
        let payload = last_watched.json::<serde_json::Value>();
        assert_eq!(payload.as_array().unwrap().len(), 1);
        assert_eq!(
            payload[0]["podcastEpisode"]["episode_id"],
            json!("watchtime-episode-1")
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_log_watchtime_for_unknown_episode_returns_not_found() {
        let server = handle_test_startup().await;

        let response = server
            .test_server
            .post("/api/v1/podcasts/episode")
            .json(&json!({
                "podcastEpisodeId": "does-not-exist",
                "time": 10
            }))
            .await;

        assert_eq!(response.status_code(), 404);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_watchtime_for_unknown_episode_returns_not_found() {
        let server = handle_test_startup().await;

        let response = server
            .test_server
            .get("/api/v1/podcasts/episode/never-seen-episode")
            .await;

        assert_eq!(response.status_code(), 404);
    }

    #[tokio::test]
    #[serial]
    async fn test_log_watchtime_rejects_invalid_payload() {
        let server = handle_test_startup().await;

        let missing_time_response = server
            .test_server
            .post("/api/v1/podcasts/episode")
            .json(&json!({
                "podcastEpisodeId": "watchtime-invalid"
            }))
            .await;
        assert_client_error_status(missing_time_response.status_code().as_u16());

        let wrong_type_response = server
            .test_server
            .post("/api/v1/podcasts/episode")
            .json(&json!({
                "podcastEpisodeId": "watchtime-invalid",
                "time": "not-a-number"
            }))
            .await;
        assert_client_error_status(wrong_type_response.status_code().as_u16());
    }

    #[tokio::test]
    #[serial]
    async fn test_get_last_watched_rewrites_urls_with_forwarded_headers() {
        let mut server = handle_test_startup().await;
        let unique = Uuid::new_v4().to_string();
        let podcast_slug = format!("watchtime-rewrite-podcast-{unique}");

        let podcast = crate::services::podcast::service::PodcastService::add_podcast_to_database(
            &format!("Watchtime Rewrite Podcast {unique}"),
            &podcast_slug,
            &format!("https://example.com/{podcast_slug}.xml"),
            "http://localhost:8080/ui/default.jpg",
            &podcast_slug,
        )
        .unwrap();
        let podcast_episode = insert_episode(
            podcast.id,
            &format!("watchtime-rewrite-episode-{unique}"),
            &format!("watchtime-rewrite-guid-{unique}"),
            "Watchtime Rewrite Episode",
        );

        let log_response = server
            .test_server
            .post("/api/v1/podcasts/episode")
            .json(&json!({
                "podcastEpisodeId": podcast_episode.episode_id,
                "time": 77
            }))
            .await;
        assert_eq!(log_response.status_code(), 200);

        server
            .test_server
            .add_header("x-forwarded-host", "podfetch.example.com");
        server.test_server.add_header("x-forwarded-proto", "https");
        server
            .test_server
            .add_header("x-forwarded-prefix", "/mobile");

        let response = server
            .test_server
            .get("/api/v1/podcasts/episode/lastwatched")
            .await;
        assert_eq!(response.status_code(), 200);

        let payload = response.json::<serde_json::Value>();
        let first_item = &payload.as_array().unwrap()[0];
        assert!(
            first_item["podcastEpisode"]["local_url"]
                .as_str()
                .unwrap()
                .starts_with("https://podfetch.example.com/mobile/")
        );
        assert!(
            first_item["podcast"]["image_url"]
                .as_str()
                .unwrap()
                .starts_with("https://podfetch.example.com/mobile/")
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_get_last_watched_returns_empty_without_watch_history() {
        let server = handle_test_startup().await;

        let response = server
            .test_server
            .get("/api/v1/podcasts/episode/lastwatched")
            .await;
        assert_eq!(response.status_code(), 200);

        let payload = response.json::<serde_json::Value>();
        assert!(payload.as_array().unwrap().is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn test_log_watchtime_twice_updates_existing_position() {
        let server = handle_test_startup().await;
        let unique = Uuid::new_v4().to_string();
        let podcast_slug = format!("watchtime-update-podcast-{unique}");
        let episode_id = format!("watchtime-update-episode-{unique}");

        let podcast = crate::services::podcast::service::PodcastService::add_podcast_to_database(
            &format!("Watchtime Update Podcast {unique}"),
            &podcast_slug,
            &format!("https://example.com/{podcast_slug}.xml"),
            "http://localhost:8080/ui/default.jpg",
            &podcast_slug,
        )
        .unwrap();
        let podcast_episode = insert_episode(
            podcast.id,
            &episode_id,
            &format!("watchtime-update-guid-{unique}"),
            "Watchtime Update Episode",
        );

        let first_log = server
            .test_server
            .post("/api/v1/podcasts/episode")
            .json(&json!({
                "podcastEpisodeId": podcast_episode.episode_id,
                "time": 40
            }))
            .await;
        assert_eq!(first_log.status_code(), 200);

        let second_log = server
            .test_server
            .post("/api/v1/podcasts/episode")
            .json(&json!({
                "podcastEpisodeId": episode_id,
                "time": 95
            }))
            .await;
        assert_eq!(second_log.status_code(), 200);

        let get_response = server
            .test_server
            .get(&format!("/api/v1/podcasts/episode/{episode_id}"))
            .await;
        assert_eq!(get_response.status_code(), 200);
        let episode = get_response.json::<EpisodeDto>();
        assert_eq!(episode.position, Some(95));
        assert_eq!(episode.started, Some(95));
    }

    #[tokio::test]
    #[serial]
    async fn test_log_watchtime_with_lower_time_keeps_endpoint_successful() {
        let server = handle_test_startup().await;
        let unique = Uuid::new_v4().to_string();
        let podcast_slug = format!("watchtime-seekback-podcast-{unique}");
        let episode_id = format!("watchtime-seekback-episode-{unique}");

        let podcast = crate::services::podcast::service::PodcastService::add_podcast_to_database(
            &format!("Watchtime Seekback Podcast {unique}"),
            &podcast_slug,
            &format!("https://example.com/{podcast_slug}.xml"),
            "http://localhost:8080/ui/default.jpg",
            &podcast_slug,
        )
        .unwrap();
        let podcast_episode = insert_episode(
            podcast.id,
            &episode_id,
            &format!("watchtime-seekback-guid-{unique}"),
            "Watchtime Seekback Episode",
        );

        let first_log = server
            .test_server
            .post("/api/v1/podcasts/episode")
            .json(&json!({
                "podcastEpisodeId": podcast_episode.episode_id,
                "time": 120
            }))
            .await;
        assert_eq!(first_log.status_code(), 200);

        let second_log = server
            .test_server
            .post("/api/v1/podcasts/episode")
            .json(&json!({
                "podcastEpisodeId": episode_id,
                "time": 60
            }))
            .await;
        assert_eq!(second_log.status_code(), 200);

        let get_response = server
            .test_server
            .get(&format!("/api/v1/podcasts/episode/{episode_id}"))
            .await;
        assert_eq!(get_response.status_code(), 200);
        let episode = get_response.json::<EpisodeDto>();
        assert_eq!(episode.position, Some(60));
        assert_eq!(episode.started, Some(60));
    }

    #[tokio::test]
    #[serial]
    async fn test_get_last_watched_rewrites_urls_with_host_fallback_only() {
        let mut server = handle_test_startup().await;
        let unique = Uuid::new_v4().to_string();
        let podcast_slug = format!("watchtime-host-fallback-podcast-{unique}");

        let podcast = crate::services::podcast::service::PodcastService::add_podcast_to_database(
            &format!("Watchtime Host Fallback Podcast {unique}"),
            &podcast_slug,
            &format!("https://example.com/{podcast_slug}.xml"),
            "http://localhost:8080/ui/default.jpg",
            &podcast_slug,
        )
        .unwrap();
        let podcast_episode = insert_episode(
            podcast.id,
            &format!("watchtime-host-fallback-episode-{unique}"),
            &format!("watchtime-host-fallback-guid-{unique}"),
            "Watchtime Host Fallback Episode",
        );

        let log_response = server
            .test_server
            .post("/api/v1/podcasts/episode")
            .json(&json!({
                "podcastEpisodeId": podcast_episode.episode_id,
                "time": 33
            }))
            .await;
        assert_eq!(log_response.status_code(), 200);

        server
            .test_server
            .add_header("x-forwarded-host", "podfetch-host-only.example.com");

        let response = server
            .test_server
            .get("/api/v1/podcasts/episode/lastwatched")
            .await;
        assert_eq!(response.status_code(), 200);

        let payload = response.json::<serde_json::Value>();
        let first_item = &payload.as_array().unwrap()[0];
        assert!(
            first_item["podcastEpisode"]["local_url"]
                .as_str()
                .unwrap()
                .contains("://podfetch-host-only.example.com/")
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_watchtime_endpoints_return_client_error_for_wrong_http_methods() {
        let server = handle_test_startup().await;

        let get_on_log_endpoint = server.test_server.get("/api/v1/podcasts/episode").await;
        assert_client_error_status(get_on_log_endpoint.status_code().as_u16());

        let put_on_log_endpoint = server.test_server.put("/api/v1/podcasts/episode").await;
        assert_client_error_status(put_on_log_endpoint.status_code().as_u16());

        let post_on_get_by_id = server
            .test_server
            .post("/api/v1/podcasts/episode/some-id")
            .await;
        assert_client_error_status(post_on_get_by_id.status_code().as_u16());

        let post_on_last_watched = server
            .test_server
            .post("/api/v1/podcasts/episode/lastwatched")
            .await;
        assert_client_error_status(post_on_last_watched.status_code().as_u16());
    }

    #[tokio::test]
    #[serial]
    async fn test_watchtime_endpoints_return_not_found_for_invalid_paths() {
        let server = handle_test_startup().await;

        let wrong_prefix = server.test_server.get("/api/v1/podcastz/episode").await;
        assert_eq!(wrong_prefix.status_code(), 404);

        let wrong_pluralization = server.test_server.get("/api/v1/podcats/episodes").await;
        assert_eq!(wrong_pluralization.status_code(), 404);

        let extra_segment = server
            .test_server
            .get("/api/v1/podcasts/episode/lastwatched/extra")
            .await;
        assert_eq!(extra_segment.status_code(), 404);
    }

    #[tokio::test]
    #[serial]
    async fn test_log_watchtime_rejects_non_object_payloads() {
        let server = handle_test_startup().await;

        let null_payload_response = server
            .test_server
            .post("/api/v1/podcasts/episode")
            .json(&json!(null))
            .await;
        assert_client_error_status(null_payload_response.status_code().as_u16());

        let array_payload_response = server
            .test_server
            .post("/api/v1/podcasts/episode")
            .json(&json!(["episode", 10]))
            .await;
        assert_client_error_status(array_payload_response.status_code().as_u16());
    }

    #[tokio::test]
    #[serial]
    async fn test_get_watchtime_with_invalid_path_fragment_returns_client_error() {
        let server = handle_test_startup().await;

        let response = server.test_server.get("/api/v1/podcasts/episode/%").await;
        assert_client_error_status(response.status_code().as_u16());
    }
}



