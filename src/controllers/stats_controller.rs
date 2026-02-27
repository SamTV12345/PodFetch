use crate::models::stats::StatsOverview;
use crate::models::user::User;
use crate::utils::error::ErrorSeverity::Info;
use crate::utils::error::{CustomError, CustomErrorInner};
use crate::utils::url_builder::{resolve_server_url_from_headers, rewrite_env_server_url_prefix};
use axum::extract::Query;
use axum::http::HeaderMap;
use axum::{Extension, Json};
use chrono::{DateTime, NaiveDate, NaiveDateTime};
use utoipa::IntoParams;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[derive(Debug, Serialize, Deserialize, Clone, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct StatsOverviewQueryParams {
    pub from: Option<String>,
    pub to: Option<String>,
    pub top_limit: Option<usize>,
}

fn parse_datetime(input: &str, end_of_day: bool) -> Result<NaiveDateTime, CustomError> {
    if let Ok(parsed) = DateTime::parse_from_rfc3339(input) {
        return Ok(parsed.naive_utc());
    }
    if let Ok(parsed_date) = NaiveDate::parse_from_str(input, "%Y-%m-%d") {
        return if end_of_day {
            Ok(parsed_date.and_hms_opt(23, 59, 59).unwrap())
        } else {
            Ok(parsed_date.and_hms_opt(0, 0, 0).unwrap())
        };
    }

    Err(CustomErrorInner::BadRequest(
        format!("Invalid datetime format: {input}. Use RFC3339 or YYYY-MM-DD."),
        Info,
    )
    .into())
}

#[utoipa::path(
get,
path="/stats/overview",
params(StatsOverviewQueryParams),
responses(
(status = 200, description = "Gets listening statistics overview for the current user.", body = StatsOverview)),
tag="stats"
)]
pub async fn get_stats_overview(
    Extension(requester): Extension<User>,
    Query(params): Query<StatsOverviewQueryParams>,
    headers: HeaderMap,
) -> Result<Json<StatsOverview>, CustomError> {
    let from = params
        .from
        .as_deref()
        .map(|from| parse_datetime(from, false))
        .transpose()?;
    let to = params
        .to
        .as_deref()
        .map(|to| parse_datetime(to, true))
        .transpose()?;
    if let (Some(from), Some(to)) = (from, to)
        && from > to
    {
        return Err(CustomErrorInner::BadRequest(
            "'from' must be less than or equal to 'to'".to_string(),
            Info,
        )
        .into());
    }

    let top_limit = params.top_limit.unwrap_or(5).clamp(1, 20);
    let server_url = resolve_server_url_from_headers(&headers);
    let mut stats = StatsOverview::calculate_for_user(&requester.username, from, to, top_limit)?;
    stats.top_podcasts.iter_mut().for_each(|podcast| {
        podcast.image_url = rewrite_env_server_url_prefix(&podcast.image_url, &server_url);
    });
    Ok(Json(stats))
}

pub fn get_stats_router() -> OpenApiRouter {
    OpenApiRouter::new().routes(routes!(get_stats_overview))
}

#[cfg(test)]
mod tests {
    use crate::adapters::persistence::dbconfig::db::get_connection;
    use crate::adapters::persistence::dbconfig::schema::podcast_episodes::dsl as pe_dsl;
    use crate::commands::startup::tests::handle_test_startup;
    use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
    use crate::models::listening_event::{ListeningEvent, NewListeningEvent};
    use crate::models::podcast_episode::PodcastEpisode;
    use crate::models::podcasts::Podcast;
    use chrono::{NaiveDate, NaiveDateTime};
    use diesel::ExpressionMethods;
    use diesel::RunQueryDsl;
    use serial_test::serial;

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

        let podcast_a = Podcast::add_podcast_to_database(
            "Stats Podcast A",
            "stats-a",
            "https://example.com/stats-a.xml",
            "http://localhost:8080/ui/default.jpg",
            "stats-a",
        )
        .unwrap();
        let podcast_b = Podcast::add_podcast_to_database(
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

        ListeningEvent::insert_event(NewListeningEvent {
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
        ListeningEvent::insert_event(NewListeningEvent {
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
        ListeningEvent::insert_event(NewListeningEvent {
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

        let podcast = Podcast::add_podcast_to_database(
            "Stats Date Range",
            "stats-date-range",
            "https://example.com/stats-date.xml",
            "http://localhost:8080/ui/default.jpg",
            "stats-date",
        )
        .unwrap();
        let episode = insert_episode(podcast.id, "stats-ep-range", "stats-guid-range", "Range");

        ListeningEvent::insert_event(NewListeningEvent {
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
        ListeningEvent::insert_event(NewListeningEvent {
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
}
