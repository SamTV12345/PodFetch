use crate::app_state::AppState;
use crate::stats::{self, StatsControllerError, StatsOverview, StatsOverviewQueryParams};
use crate::url_rewriting::create_url_rewriter;
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::{Extension, Json};
use common_infrastructure::error::ErrorSeverity::Info;
use common_infrastructure::error::{CustomError, CustomErrorInner};
use podfetch_domain::user::User;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[utoipa::path(
get,
path="/stats/overview",
params(StatsOverviewQueryParams),
responses(
(status = 200, description = "Gets listening statistics overview for the current user.", body = StatsOverview)),
tag="stats"
)]
pub async fn get_stats_overview(
    State(state): State<AppState>,
    Extension(requester): Extension<User>,
    Query(params): Query<StatsOverviewQueryParams>,
    headers: HeaderMap,
) -> Result<Json<StatsOverview>, CustomError> {
    let rewriter = create_url_rewriter(&headers);
    let mut stats =
        stats::get_stats_overview(state.stats_service.as_ref(), &requester.username, params)
            .map_err(map_stats_error)?;
    stats.top_podcasts.iter_mut().for_each(|podcast| {
        rewriter.rewrite_in_place(&mut podcast.image_url);
    });
    Ok(Json(stats))
}

fn map_stats_error(error: StatsControllerError<CustomError>) -> CustomError {
    match error {
        StatsControllerError::BadRequest(message) => {
            CustomErrorInner::BadRequest(message, Info).into()
        }
        StatsControllerError::Service(error) => error,
    }
}

pub fn get_stats_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(get_stats_overview))
}

#[cfg(test)]
mod tests {
    use crate::services::listening_event::service::ListeningEventService;
    use crate::test_support::tests::handle_test_startup;
    use chrono::{NaiveDate, NaiveDateTime};
    use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
    use diesel::ExpressionMethods;
    use diesel::RunQueryDsl;
    use podfetch_domain::listening_event::NewListeningEvent;
    use podfetch_persistence::db::get_connection;
    use podfetch_persistence::podcast_episode::PodcastEpisodeEntity as PodcastEpisode;
    use podfetch_persistence::schema::podcast_episodes::dsl as pe_dsl;
    use serde::Deserialize;
    use serde_json::Value;
    use serial_test::serial;
    use uuid::Uuid;

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct TopPodcastStatsResponse {
        podcast_id: i32,
        podcast_name: String,
        listened_seconds: i64,
        listened_episodes: i64,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct WeekdayStatsResponse {
        weekday: String,
        listened_seconds: i64,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct StatsOverviewResponse {
        listened_podcasts: i64,
        listened_episodes: i64,
        total_listened_seconds: i64,
        top_podcasts: Vec<TopPodcastStatsResponse>,
        active_weekdays: Vec<WeekdayStatsResponse>,
    }

    fn dt(day: u32, hour: u32, minute: u32, second: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 2, day)
            .unwrap()
            .and_hms_opt(hour, minute, second)
            .unwrap()
    }

    fn unique_name(prefix: &str) -> String {
        format!("{prefix}-{}", Uuid::new_v4())
    }

    fn assert_client_error_status(status: u16) {
        assert!(
            (400..500).contains(&status),
            "expected 4xx status, got {status}"
        );
    }

    fn insert_episode(
        podcast_id: i32,
        episode_id: &str,
        guid: &str,
        title: &str,
    ) -> PodcastEpisode {
        diesel::insert_into(pe_dsl::podcast_episodes)
            .values((
                pe_dsl::podcast_id.eq(podcast_id),
                pe_dsl::episode_id.eq(episode_id.to_string()),
                pe_dsl::name.eq(title.to_string()),
                pe_dsl::url.eq(format!("https://example.com/{episode_id}.mp3")),
                pe_dsl::date_of_recording.eq("2026-02-27T00:00:00Z".to_string()),
                pe_dsl::image_url.eq("http://localhost:8080/ui/default.jpg".to_string()),
                pe_dsl::total_time.eq(1800),
                pe_dsl::description.eq("test description".to_string()),
                pe_dsl::guid.eq(guid.to_string()),
                pe_dsl::deleted.eq(false),
                pe_dsl::episode_numbering_processed.eq(false),
            ))
            .get_result::<PodcastEpisode>(&mut get_connection())
            .unwrap()
    }

    #[tokio::test]
    #[serial]
    async fn test_get_stats_overview_aggregates_data() {
        let server = handle_test_startup().await;
        let username = ENVIRONMENT_SERVICE
            .username
            .clone()
            .unwrap_or_else(|| "user123".to_string());

        let podcast_a = crate::services::podcast::service::PodcastService::add_podcast_to_database(
            "Stats Podcast A",
            "stats-a",
            "https://example.com/stats-a.xml",
            "http://localhost:8080/ui/default.jpg",
            "stats-a",
        )
        .unwrap();
        let podcast_b = crate::services::podcast::service::PodcastService::add_podcast_to_database(
            "Stats Podcast B",
            "stats-b",
            "https://example.com/stats-b.xml",
            "http://localhost:8080/ui/default.jpg",
            "stats-b",
        )
        .unwrap();

        let episode_a1 = insert_episode(podcast_a.id, "stats-ep-a1", "stats-guid-a1", "A1");
        let episode_a2 = insert_episode(podcast_a.id, "stats-ep-a2", "stats-guid-a2", "A2");
        let episode_b1 = insert_episode(podcast_b.id, "stats-ep-b1", "stats-guid-b1", "B1");

        ListeningEventService::create_event(NewListeningEvent {
            username: username.clone(),
            device: "webview".to_string(),
            podcast_episode_id: episode_a1.episode_id.clone(),
            podcast_id: podcast_a.id,
            podcast_episode_db_id: episode_a1.id,
            delta_seconds: 120,
            start_position: 10,
            end_position: 130,
            listened_at: dt(23, 10, 0, 0),
        })
        .unwrap();
        ListeningEventService::create_event(NewListeningEvent {
            username: username.clone(),
            device: "webview".to_string(),
            podcast_episode_id: episode_a2.episode_id.clone(),
            podcast_id: podcast_a.id,
            podcast_episode_db_id: episode_a2.id,
            delta_seconds: 240,
            start_position: 0,
            end_position: 240,
            listened_at: dt(24, 11, 0, 0),
        })
        .unwrap();
        ListeningEventService::create_event(NewListeningEvent {
            username,
            device: "webview".to_string(),
            podcast_episode_id: episode_b1.episode_id.clone(),
            podcast_id: podcast_b.id,
            podcast_episode_db_id: episode_b1.id,
            delta_seconds: 180,
            start_position: 100,
            end_position: 280,
            listened_at: dt(25, 12, 0, 0),
        })
        .unwrap();

        let response = server.test_server.get("/api/v1/stats/overview").await;
        assert_eq!(response.status_code(), 200);

        let payload = response.json::<StatsOverviewResponse>();
        assert_eq!(payload.listened_podcasts, 2);
        assert_eq!(payload.listened_episodes, 3);
        assert_eq!(payload.total_listened_seconds, 540);
        assert_eq!(payload.top_podcasts.len(), 2);
        assert_eq!(payload.top_podcasts[0].podcast_name, "Stats Podcast A");
        assert_eq!(payload.top_podcasts[0].podcast_id, podcast_a.id);
        assert_eq!(payload.top_podcasts[0].listened_seconds, 360);
        assert_eq!(payload.top_podcasts[0].listened_episodes, 2);

        let wednesday = payload
            .active_weekdays
            .iter()
            .find(|weekday| weekday.weekday == "wednesday")
            .unwrap();
        assert_eq!(wednesday.listened_seconds, 180);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_stats_overview_respects_date_range() {
        let server = handle_test_startup().await;
        let username = ENVIRONMENT_SERVICE
            .username
            .clone()
            .unwrap_or_else(|| "user123".to_string());

        let podcast = crate::services::podcast::service::PodcastService::add_podcast_to_database(
            "Stats Date Range",
            "stats-date-range",
            "https://example.com/stats-date.xml",
            "http://localhost:8080/ui/default.jpg",
            "stats-date",
        )
        .unwrap();
        let episode = insert_episode(podcast.id, "stats-ep-range", "stats-guid-range", "Range");

        ListeningEventService::create_event(NewListeningEvent {
            username: username.clone(),
            device: "webview".to_string(),
            podcast_episode_id: episode.episode_id.clone(),
            podcast_id: podcast.id,
            podcast_episode_db_id: episode.id,
            delta_seconds: 300,
            start_position: 0,
            end_position: 300,
            listened_at: dt(24, 10, 0, 0),
        })
        .unwrap();
        ListeningEventService::create_event(NewListeningEvent {
            username,
            device: "webview".to_string(),
            podcast_episode_id: episode.episode_id,
            podcast_id: podcast.id,
            podcast_episode_db_id: episode.id,
            delta_seconds: 60,
            start_position: 300,
            end_position: 360,
            listened_at: dt(25, 10, 0, 0),
        })
        .unwrap();

        let response = server
            .test_server
            .get("/api/v1/stats/overview?from=2026-02-25&to=2026-02-25")
            .await;
        assert_eq!(response.status_code(), 200);

        let payload = response.json::<StatsOverviewResponse>();
        assert_eq!(payload.total_listened_seconds, 60);
        assert_eq!(payload.listened_podcasts, 1);
        assert_eq!(payload.listened_episodes, 1);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_stats_overview_rejects_invalid_datetime_query() {
        let server = handle_test_startup().await;

        let response = server
            .test_server
            .get("/api/v1/stats/overview?from=not-a-date")
            .await;

        assert_client_error_status(response.status_code().as_u16());
    }

    #[tokio::test]
    #[serial]
    async fn test_get_stats_overview_rejects_from_after_to() {
        let server = handle_test_startup().await;

        let response = server
            .test_server
            .get("/api/v1/stats/overview?from=2026-02-26&to=2026-02-25")
            .await;

        assert_client_error_status(response.status_code().as_u16());
    }

    #[tokio::test]
    #[serial]
    async fn test_get_stats_overview_clamps_top_limit_to_maximum() {
        let server = handle_test_startup().await;
        let username = ENVIRONMENT_SERVICE
            .username
            .clone()
            .unwrap_or_else(|| "user123".to_string());

        for i in 0..3 {
            let unique = Uuid::new_v4().to_string();
            let podcast_slug = format!("stats-top-limit-{i}-{unique}");
            let podcast =
                crate::services::podcast::service::PodcastService::add_podcast_to_database(
                    &unique_name("Stats Top Limit Podcast"),
                    &podcast_slug,
                    &format!("https://example.com/{podcast_slug}.xml"),
                    "http://localhost:8080/ui/default.jpg",
                    &podcast_slug,
                )
                .unwrap();
            let episode = insert_episode(
                podcast.id,
                &format!("stats-top-limit-ep-{i}-{unique}"),
                &format!("stats-top-limit-guid-{i}-{unique}"),
                "Top Limit Episode",
            );

            ListeningEventService::create_event(NewListeningEvent {
                username: username.clone(),
                device: "webview".to_string(),
                podcast_episode_id: episode.episode_id,
                podcast_id: podcast.id,
                podcast_episode_db_id: episode.id,
                delta_seconds: 30,
                start_position: 0,
                end_position: 30,
                listened_at: dt(24, 8 + i, 0, 0),
            })
            .unwrap();
        }

        let response = server
            .test_server
            .get("/api/v1/stats/overview?topLimit=999")
            .await;
        assert_eq!(response.status_code(), 200);

        let payload = response.json::<StatsOverviewResponse>();
        assert!(payload.top_podcasts.len() <= 20);
        assert_eq!(payload.top_podcasts.len(), 3);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_stats_overview_rejects_invalid_to_datetime_query() {
        let server = handle_test_startup().await;

        let response = server
            .test_server
            .get("/api/v1/stats/overview?to=not-a-date")
            .await;

        assert_client_error_status(response.status_code().as_u16());
    }

    #[tokio::test]
    #[serial]
    async fn test_get_stats_overview_accepts_rfc3339_range() {
        let server = handle_test_startup().await;
        let username = ENVIRONMENT_SERVICE
            .username
            .clone()
            .unwrap_or_else(|| "user123".to_string());
        let unique = Uuid::new_v4().to_string();
        let podcast_slug = format!("stats-rfc3339-podcast-{unique}");

        let podcast = crate::services::podcast::service::PodcastService::add_podcast_to_database(
            &unique_name("Stats RFC3339 Podcast"),
            &podcast_slug,
            &format!("https://example.com/{podcast_slug}.xml"),
            "http://localhost:8080/ui/default.jpg",
            &podcast_slug,
        )
        .unwrap();
        let episode = insert_episode(
            podcast.id,
            &format!("stats-rfc3339-episode-{unique}"),
            &format!("stats-rfc3339-guid-{unique}"),
            "Stats RFC3339 Episode",
        );

        ListeningEventService::create_event(NewListeningEvent {
            username,
            device: "webview".to_string(),
            podcast_episode_id: episode.episode_id,
            podcast_id: podcast.id,
            podcast_episode_db_id: episode.id,
            delta_seconds: 90,
            start_position: 0,
            end_position: 90,
            listened_at: dt(24, 10, 0, 0),
        })
        .unwrap();

        let response = server
            .test_server
            .get("/api/v1/stats/overview?from=2026-02-24T00:00:00Z&to=2026-02-24T23:59:59Z")
            .await;
        assert_eq!(response.status_code(), 200);

        let payload = response.json::<StatsOverviewResponse>();
        assert_eq!(payload.total_listened_seconds, 90);
        assert_eq!(payload.listened_podcasts, 1);
        assert_eq!(payload.listened_episodes, 1);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_stats_overview_clamps_top_limit_to_minimum_one() {
        let server = handle_test_startup().await;
        let username = ENVIRONMENT_SERVICE
            .username
            .clone()
            .unwrap_or_else(|| "user123".to_string());

        for i in 0..2 {
            let unique = Uuid::new_v4().to_string();
            let podcast_slug = format!("stats-min-top-limit-{i}-{unique}");
            let podcast =
                crate::services::podcast::service::PodcastService::add_podcast_to_database(
                    &unique_name("Stats Min Top Limit Podcast"),
                    &podcast_slug,
                    &format!("https://example.com/{podcast_slug}.xml"),
                    "http://localhost:8080/ui/default.jpg",
                    &podcast_slug,
                )
                .unwrap();
            let episode = insert_episode(
                podcast.id,
                &format!("stats-min-top-limit-episode-{i}-{unique}"),
                &format!("stats-min-top-limit-guid-{i}-{unique}"),
                "Stats Min Top Limit Episode",
            );

            ListeningEventService::create_event(NewListeningEvent {
                username: username.clone(),
                device: "webview".to_string(),
                podcast_episode_id: episode.episode_id,
                podcast_id: podcast.id,
                podcast_episode_db_id: episode.id,
                delta_seconds: 30 + (i * 10),
                start_position: 0,
                end_position: 40,
                listened_at: dt(24 + i as u32, 9, 0, 0),
            })
            .unwrap();
        }

        let response = server
            .test_server
            .get("/api/v1/stats/overview?topLimit=0")
            .await;
        assert_eq!(response.status_code(), 200);

        let payload = response.json::<StatsOverviewResponse>();
        assert_eq!(payload.top_podcasts.len(), 1);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_stats_overview_rewrites_image_urls_with_forwarded_headers() {
        let mut server = handle_test_startup().await;
        let username = ENVIRONMENT_SERVICE
            .username
            .clone()
            .unwrap_or_else(|| "user123".to_string());
        let unique = Uuid::new_v4().to_string();
        let podcast_slug = format!("stats-rewrite-podcast-{unique}");

        let podcast = crate::services::podcast::service::PodcastService::add_podcast_to_database(
            &unique_name("Stats Rewrite Podcast"),
            &podcast_slug,
            &format!("https://example.com/{podcast_slug}.xml"),
            "http://localhost:8080/ui/default.jpg",
            &podcast_slug,
        )
        .unwrap();
        let episode = insert_episode(
            podcast.id,
            &format!("stats-rewrite-episode-{unique}"),
            &format!("stats-rewrite-guid-{unique}"),
            "Stats Rewrite Episode",
        );

        ListeningEventService::create_event(NewListeningEvent {
            username,
            device: "webview".to_string(),
            podcast_episode_id: episode.episode_id,
            podcast_id: podcast.id,
            podcast_episode_db_id: episode.id,
            delta_seconds: 42,
            start_position: 0,
            end_position: 42,
            listened_at: dt(24, 8, 0, 0),
        })
        .unwrap();

        server
            .test_server
            .add_header("x-forwarded-host", "podfetch.example.com");
        server.test_server.add_header("x-forwarded-proto", "https");
        server.test_server.add_header("x-forwarded-prefix", "/ui");

        let response = server.test_server.get("/api/v1/stats/overview").await;
        assert_eq!(response.status_code(), 200);

        let payload = response.json::<Value>();
        let top_podcasts = payload["topPodcasts"].as_array().unwrap();
        assert!(!top_podcasts.is_empty());
        assert!(
            top_podcasts[0]["imageUrl"]
                .as_str()
                .unwrap()
                .starts_with("https://podfetch.example.com/ui/")
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_get_stats_overview_returns_client_error_for_wrong_http_methods() {
        let server = handle_test_startup().await;

        let post_response = server.test_server.post("/api/v1/stats/overview").await;
        assert_client_error_status(post_response.status_code().as_u16());

        let put_response = server.test_server.put("/api/v1/stats/overview").await;
        assert_client_error_status(put_response.status_code().as_u16());

        let delete_response = server.test_server.delete("/api/v1/stats/overview").await;
        assert_client_error_status(delete_response.status_code().as_u16());
    }

    #[tokio::test]
    #[serial]
    async fn test_stats_endpoints_return_not_found_for_invalid_paths() {
        let server = handle_test_startup().await;

        let wrong_base = server.test_server.get("/api/v1/stat/overview").await;
        assert_eq!(wrong_base.status_code(), 404);

        let typo_path = server.test_server.get("/api/v1/stats/overviews").await;
        assert_eq!(typo_path.status_code(), 404);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_stats_overview_rejects_trailing_slash_path_variant() {
        let server = handle_test_startup().await;

        let response = server.test_server.get("/api/v1/stats/overview/").await;
        assert_client_error_status(response.status_code().as_u16());
    }

    #[tokio::test]
    #[serial]
    async fn test_get_stats_overview_rejects_invalid_top_limit_query_values() {
        let server = handle_test_startup().await;

        let non_numeric_response = server
            .test_server
            .get("/api/v1/stats/overview?topLimit=abc")
            .await;
        assert_client_error_status(non_numeric_response.status_code().as_u16());

        let negative_response = server
            .test_server
            .get("/api/v1/stats/overview?topLimit=-1")
            .await;
        assert_client_error_status(negative_response.status_code().as_u16());

        let overflow_response = server
            .test_server
            .get("/api/v1/stats/overview?topLimit=999999999999999999999999")
            .await;
        assert_client_error_status(overflow_response.status_code().as_u16());
    }

    #[tokio::test]
    #[serial]
    async fn test_get_stats_overview_rejects_empty_date_query_values() {
        let server = handle_test_startup().await;

        let empty_from_response = server.test_server.get("/api/v1/stats/overview?from=").await;
        assert_client_error_status(empty_from_response.status_code().as_u16());

        let empty_to_response = server.test_server.get("/api/v1/stats/overview?to=").await;
        assert_client_error_status(empty_to_response.status_code().as_u16());
    }
}
