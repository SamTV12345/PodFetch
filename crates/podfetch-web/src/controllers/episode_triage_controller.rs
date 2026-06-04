use crate::app_state::AppState;
use crate::controllers::podcast_episode_controller::{
    PodcastEpisodeWithHistory, resolve_episode_uuid, spawn_single_episode_download,
};
use crate::services::episode_triage::service::DEFAULT_PAGE_SIZE;
use crate::url_rewriting::resolve_server_url_from_headers;
use crate::usecases::podcast_episode::PodcastEpisodeUseCase as PodcastEpisodeService;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::{Extension, Json};
use common_infrastructure::error::ErrorSeverity::Warning;
use common_infrastructure::error::{CustomError, CustomErrorInner};
use podfetch_domain::episode_triage::TriageStatus;
use podfetch_domain::user::User;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

const MAX_PAGE_SIZE: i64 = 100;

#[derive(Debug, Serialize, Deserialize, Clone, IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct TriageListQuery {
    /// Exclusive `date_of_recording` cursor for keyset pagination. Pass the
    /// `date_of_recording` of the last item from the previous page.
    pub last_episode_date: Option<String>,
    /// Page size. Defaults to 30, capped at 100.
    pub limit: Option<i64>,
}

impl TriageListQuery {
    fn limit(&self) -> i64 {
        self.limit.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct TriageStatusPut {
    /// One of `queued`, `archived` or `dismissed`.
    pub status: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ClearInboxResponse {
    pub dismissed: usize,
}

#[utoipa::path(
    get,
    path = "/episodes/inbox",
    params(TriageListQuery),
    responses((status = 200, description = "New, not-yet-triaged episodes (the inbox).", body = [PodcastEpisodeWithHistory])),
    tag = "podcast_episodes"
)]
pub async fn get_inbox(
    State(state): State<AppState>,
    Extension(requester): Extension<User>,
    Query(query): Query<TriageListQuery>,
    headers: HeaderMap,
) -> Result<Json<Vec<PodcastEpisodeWithHistory>>, CustomError> {
    let server_url = resolve_server_url_from_headers(&headers);
    let items = state.episode_triage_service.get_inbox(
        &requester,
        query.last_episode_date.clone(),
        query.limit(),
        &server_url,
    )?;
    Ok(Json(items))
}

#[utoipa::path(
    get,
    path = "/episodes/waiting-list",
    responses((status = 200, description = "Episodes the user picked to listen to.", body = [PodcastEpisodeWithHistory])),
    tag = "podcast_episodes"
)]
pub async fn get_waiting_list(
    State(state): State<AppState>,
    Extension(requester): Extension<User>,
    headers: HeaderMap,
) -> Result<Json<Vec<PodcastEpisodeWithHistory>>, CustomError> {
    let server_url = resolve_server_url_from_headers(&headers);
    let items = state
        .episode_triage_service
        .get_waiting_list(&requester, &server_url)?;
    Ok(Json(items))
}

#[utoipa::path(
    get,
    path = "/episodes/archive",
    params(TriageListQuery),
    responses((status = 200, description = "Every downloaded episode (the archive).", body = [PodcastEpisodeWithHistory])),
    tag = "podcast_episodes"
)]
pub async fn get_archive(
    State(state): State<AppState>,
    Extension(requester): Extension<User>,
    Query(query): Query<TriageListQuery>,
    headers: HeaderMap,
) -> Result<Json<Vec<PodcastEpisodeWithHistory>>, CustomError> {
    let server_url = resolve_server_url_from_headers(&headers);
    let items = state.episode_triage_service.get_archive(
        &requester,
        query.last_episode_date.clone(),
        query.limit(),
        &server_url,
    )?;
    Ok(Json(items))
}

/// `id` is the episode id (uuid or legacy integer).
#[utoipa::path(
    put,
    path = "/episodes/{id}/triage",
    responses((status = 200, description = "Records the triage decision for an episode.")),
    tag = "podcast_episodes"
)]
pub async fn set_episode_triage(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Extension(requester): Extension<User>,
    Json(body): Json<TriageStatusPut>,
) -> Result<StatusCode, CustomError> {
    let status = TriageStatus::from_string(&body.status).ok_or_else(|| -> CustomError {
        CustomErrorInner::BadRequest(
            format!("'{}' is not a valid triage status", body.status),
            Warning,
        )
        .into()
    })?;
    let episode_uuid = resolve_episode_uuid(&id)?;
    let episode = PodcastEpisodeService::get_podcast_episode_by_internal_id(episode_uuid)?
        .ok_or_else(|| -> CustomError { CustomErrorInner::NotFound(Warning).into() })?;

    state
        .episode_triage_service
        .set_status(requester.id, episode_uuid, status)?;

    // Picking an episode from the inbox also kicks off its download — but only
    // when the user is allowed to trigger downloads and the file isn't present
    // yet. Other users still get the queued state recorded for their waiting
    // list; an admin's download then makes it playable for everyone.
    if status == TriageStatus::Queued && requester.is_privileged_user() && !episode.is_downloaded()
    {
        spawn_single_episode_download(episode.episode_id);
    }

    Ok(StatusCode::OK)
}

#[utoipa::path(
    post,
    path = "/episodes/inbox/clear",
    responses((status = 200, description = "Dismisses every episode currently in the inbox.", body = ClearInboxResponse)),
    tag = "podcast_episodes"
)]
pub async fn clear_inbox(
    State(state): State<AppState>,
    Extension(requester): Extension<User>,
) -> Result<Json<ClearInboxResponse>, CustomError> {
    let dismissed = state.episode_triage_service.clear_inbox(&requester)?;
    Ok(Json(ClearInboxResponse { dismissed }))
}

pub fn get_episode_triage_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_inbox))
        .routes(routes!(get_waiting_list))
        .routes(routes!(get_archive))
        .routes(routes!(clear_inbox))
        .routes(routes!(set_episode_triage))
}

#[cfg(test)]
mod tests {
    use super::TriageStatusPut;
    use crate::app_state::AppState;
    use crate::test_support::tests::handle_test_startup;
    use crate::test_utils::test_builder::user_test_builder::tests::UserTestDataBuilder;
    use axum::Json;
    use axum::extract::{Path, State};
    use axum::Extension;
    use chrono::Utc;
    use diesel::ExpressionMethods;
    use diesel::QueryDsl;
    use diesel::RunQueryDsl;
    use podfetch_domain::user::User;
    use podfetch_persistence::db::get_connection;
    use podfetch_persistence::podcast_episode::PodcastEpisodeEntity as PodcastEpisode;
    use podfetch_persistence::schema::podcast_episodes::dsl as pe_dsl;
    use serde_json::{Value, json};
    use serial_test::serial;
    use uuid::Uuid;

    fn unique_name(prefix: &str) -> String {
        format!("{prefix}-{}", Uuid::new_v4())
    }

    fn app_state() -> AppState {
        AppState::new()
    }

    fn non_admin_user() -> User {
        let mut user = UserTestDataBuilder::new().build();
        user.id = Uuid::new_v4();
        user.role = "user".to_string();
        user
    }

    fn add_podcast(slug: &str) -> podfetch_persistence::podcast::PodcastEntity {
        crate::services::podcast::service::PodcastService::add_podcast_to_database(
            &unique_name("Triage Podcast"),
            slug,
            &format!("https://example.com/{slug}.xml"),
            "http://localhost:8080/ui/default.jpg",
            slug,
        )
        .unwrap()
    }

    fn insert_episode(podcast_id: &str, episode_id: &str, guid: &str) -> PodcastEpisode {
        diesel::insert_into(pe_dsl::podcast_episodes)
            .values((
                pe_dsl::id.eq(podfetch_domain::ids::new_id().to_string()),
                pe_dsl::podcast_id.eq(podcast_id.to_string()),
                pe_dsl::episode_id.eq(episode_id.to_string()),
                pe_dsl::name.eq("Triage Episode".to_string()),
                pe_dsl::url.eq(format!("https://example.com/{episode_id}.mp3")),
                pe_dsl::date_of_recording.eq("2026-03-01T00:00:00Z".to_string()),
                pe_dsl::image_url.eq("http://localhost:8080/ui/default.jpg".to_string()),
                pe_dsl::total_time.eq(1800),
                pe_dsl::description.eq("triage test".to_string()),
                pe_dsl::guid.eq(guid.to_string()),
                pe_dsl::deleted.eq(false),
                pe_dsl::episode_numbering_processed.eq(false),
            ))
            .get_result::<PodcastEpisode>(&mut get_connection())
            .unwrap()
    }

    fn mark_downloaded(episode: &PodcastEpisode) {
        diesel::update(pe_dsl::podcast_episodes.filter(pe_dsl::id.eq(episode.id.clone())))
            .set((
                pe_dsl::download_location.eq("Local".to_string()),
                pe_dsl::download_time.eq(Some(Utc::now().naive_utc())),
                pe_dsl::file_episode_path.eq(format!("./podcasts/{}.mp3", episode.episode_id)),
            ))
            .execute(&mut get_connection())
            .unwrap();
    }

    fn contains_episode(payload: &Value, episode: &PodcastEpisode) -> bool {
        payload
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["podcastEpisode"]["id"] == json!(episode.id.clone()))
    }

    #[tokio::test]
    #[serial]
    async fn test_inbox_contains_untriaged_episodes_regardless_of_download_state() {
        // The inbox captures every new (untriaged, non-deleted) episode, whether
        // or not it has been downloaded. Uses a fresh user + a large limit so
        // the assertions are robust against episodes left by other tests in the
        // shared database.
        let server = handle_test_startup().await;
        let _guard = &server.mutex;
        let unique = Uuid::new_v4();
        let podcast = add_podcast(&format!("inbox-state-podcast-{unique}"));
        let pending = insert_episode(
            &podcast.id,
            &format!("inbox-pending-{unique}"),
            &format!("inbox-pending-guid-{unique}"),
        );
        let downloaded = insert_episode(
            &podcast.id,
            &format!("inbox-downloaded-{unique}"),
            &format!("inbox-downloaded-guid-{unique}"),
        );
        mark_downloaded(&downloaded);
        let dismissed = insert_episode(
            &podcast.id,
            &format!("inbox-dismissed-{unique}"),
            &format!("inbox-dismissed-guid-{unique}"),
        );

        let user = non_admin_user();
        let state = app_state();
        state
            .episode_triage_service
            .set_status(
                user.id,
                Uuid::parse_str(&dismissed.id).unwrap(),
                podfetch_domain::episode_triage::TriageStatus::Dismissed,
            )
            .unwrap();

        let inbox = state
            .episode_triage_service
            .get_inbox(&user, None, 100_000, "")
            .unwrap();
        let in_inbox = |ep: &PodcastEpisode| inbox.iter().any(|i| i.podcast_episode.id == ep.id);

        assert!(in_inbox(&pending), "undownloaded untriaged episode should be in the inbox");
        assert!(
            in_inbox(&downloaded),
            "downloaded but untriaged episode should also be in the inbox"
        );
        assert!(
            !in_inbox(&dismissed),
            "an episode the user dismissed must not be in the inbox"
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_dismiss_removes_episode_from_inbox() {
        let server = handle_test_startup().await;
        let unique = Uuid::new_v4();
        let podcast = add_podcast(&format!("dismiss-podcast-{unique}"));
        let episode = insert_episode(
            &podcast.id,
            &format!("dismiss-ep-{unique}"),
            &format!("dismiss-guid-{unique}"),
        );

        let dismiss = server
            .test_server
            .put(&format!("/api/v1/episodes/{}/triage", episode.id))
            .json(&json!({ "status": "dismissed" }))
            .await;
        assert_eq!(dismiss.status_code(), 200);

        let inbox = server.test_server.get("/api/v1/episodes/inbox").await;
        assert!(
            !contains_episode(&inbox.json::<Value>(), &episode),
            "dismissed episode should leave the inbox"
        );
        let waiting = server.test_server.get("/api/v1/episodes/waiting-list").await;
        assert!(
            !contains_episode(&waiting.json::<Value>(), &episode),
            "dismissed episode should not be in the waiting list"
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_queue_downloaded_episode_appears_in_waiting_list() {
        let server = handle_test_startup().await;
        let unique = Uuid::new_v4();
        let podcast = add_podcast(&format!("queue-podcast-{unique}"));
        let episode = insert_episode(
            &podcast.id,
            &format!("queue-ep-{unique}"),
            &format!("queue-guid-{unique}"),
        );
        // Already downloaded, so queueing must not kick off a network download.
        mark_downloaded(&episode);

        let queue = server
            .test_server
            .put(&format!("/api/v1/episodes/{}/triage", episode.id))
            .json(&json!({ "status": "queued" }))
            .await;
        assert_eq!(queue.status_code(), 200);

        let waiting = server.test_server.get("/api/v1/episodes/waiting-list").await;
        assert_eq!(waiting.status_code(), 200);
        assert!(
            contains_episode(&waiting.json::<Value>(), &episode),
            "queued episode should appear in the waiting list"
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_queue_undownloaded_episode_moves_out_of_inbox_without_download() {
        // Use a non-privileged user via a direct handler call so the download is
        // not triggered (and no network is hit) while still exercising the
        // queued path for a not-yet-downloaded episode.
        let server = handle_test_startup().await;
        let _guard = &server.mutex;
        let unique = Uuid::new_v4();
        let podcast = add_podcast(&format!("queue-pending-podcast-{unique}"));
        let episode = insert_episode(
            &podcast.id,
            &format!("queue-pending-ep-{unique}"),
            &format!("queue-pending-guid-{unique}"),
        );
        let user = non_admin_user();

        let result = super::set_episode_triage(
            State(app_state()),
            Path(episode.id.clone()),
            Extension(user.clone()),
            Json(TriageStatusPut {
                status: "queued".to_string(),
            }),
        )
        .await;
        assert!(result.is_ok());

        let waiting = app_state()
            .episode_triage_service
            .get_waiting_list(&user, "")
            .unwrap();
        assert!(
            waiting.iter().any(|i| i.podcast_episode.id == episode.id),
            "queued episode should be in the user's waiting list"
        );
        let inbox = app_state()
            .episode_triage_service
            .get_inbox(&user, None, 100, "")
            .unwrap();
        assert!(
            !inbox.iter().any(|i| i.podcast_episode.id == episode.id),
            "queued episode should no longer be in the inbox"
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_archive_lists_downloaded_excludes_undownloaded() {
        let server = handle_test_startup().await;
        let unique = Uuid::new_v4();
        let podcast = add_podcast(&format!("archive-podcast-{unique}"));
        let downloaded = insert_episode(
            &podcast.id,
            &format!("archive-dl-{unique}"),
            &format!("archive-dl-guid-{unique}"),
        );
        mark_downloaded(&downloaded);
        let pending = insert_episode(
            &podcast.id,
            &format!("archive-pending-{unique}"),
            &format!("archive-pending-guid-{unique}"),
        );

        let response = server
            .test_server
            .get("/api/v1/episodes/archive?limit=100")
            .await;
        assert_eq!(response.status_code(), 200);
        let payload = response.json::<Value>();
        assert!(
            contains_episode(&payload, &downloaded),
            "downloaded episode should be in the archive"
        );
        assert!(
            !contains_episode(&payload, &pending),
            "undownloaded episode must not be in the archive"
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_clear_inbox_dismisses_current_inbox() {
        let server = handle_test_startup().await;
        let unique = Uuid::new_v4();
        let podcast = add_podcast(&format!("clear-podcast-{unique}"));
        let first = insert_episode(
            &podcast.id,
            &format!("clear-ep-1-{unique}"),
            &format!("clear-guid-1-{unique}"),
        );
        let second = insert_episode(
            &podcast.id,
            &format!("clear-ep-2-{unique}"),
            &format!("clear-guid-2-{unique}"),
        );

        let clear = server.test_server.post("/api/v1/episodes/inbox/clear").await;
        assert_eq!(clear.status_code(), 200);
        let dismissed = clear.json::<Value>()["dismissed"].as_u64().unwrap();
        assert!(dismissed >= 2, "clear should dismiss at least the two new episodes");

        let inbox = server.test_server.get("/api/v1/episodes/inbox").await;
        let payload = inbox.json::<Value>();
        assert!(!contains_episode(&payload, &first));
        assert!(!contains_episode(&payload, &second));
    }

    #[tokio::test]
    #[serial]
    async fn test_set_triage_rejects_invalid_status() {
        let server = handle_test_startup().await;
        let unique = Uuid::new_v4();
        let podcast = add_podcast(&format!("invalid-status-podcast-{unique}"));
        let episode = insert_episode(
            &podcast.id,
            &format!("invalid-status-ep-{unique}"),
            &format!("invalid-status-guid-{unique}"),
        );

        let response = server
            .test_server
            .put(&format!("/api/v1/episodes/{}/triage", episode.id))
            .json(&json!({ "status": "banana" }))
            .await;
        assert_eq!(response.status_code(), 400);
    }

    #[tokio::test]
    #[serial]
    async fn test_set_triage_unknown_episode_returns_not_found() {
        let server = handle_test_startup().await;

        let response = server
            .test_server
            .put(&format!("/api/v1/episodes/{}/triage", Uuid::new_v4()))
            .json(&json!({ "status": "queued" }))
            .await;
        assert_eq!(response.status_code(), 404);
    }

    #[tokio::test]
    #[serial]
    async fn test_set_triage_malformed_episode_id_returns_bad_request() {
        let server = handle_test_startup().await;

        let response = server
            .test_server
            .put("/api/v1/episodes/does-not-exist/triage")
            .json(&json!({ "status": "queued" }))
            .await;
        assert_eq!(response.status_code(), 400);
    }
}
