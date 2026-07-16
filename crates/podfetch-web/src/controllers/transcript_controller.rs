//! HTTP surface for Podcasting 2.0 transcript support: listing an episode's
//! transcripts, fetching the preferred one (with segments), streaming a
//! transcript's archived file (session auth or apiKey-in-path for feed
//! clients), enqueueing a Whisper-generated transcript job, full-text search
//! across transcript segments, and an admin-only reparse-all action.

use crate::app_state::AppState;
use crate::controllers::podcast_episode_controller::resolve_episode_uuid;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Json, Response};
use axum::{Extension, http};
use common_infrastructure::error::ErrorSeverity::Warning;
use common_infrastructure::error::{CustomError, CustomErrorInner};
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
use podfetch_domain::podcast_episode_transcript::PodcastEpisodeTranscript;
use podfetch_domain::podcast_episode_transcript::TranscriptSegment;
use podfetch_domain::user::User;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

// ── DTOs ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptDto {
    pub id: String,
    pub source: String,
    pub language: Option<String>,
    pub mime_type: String,
    pub status: String,
    pub error: Option<String>,
}

impl From<PodcastEpisodeTranscript> for TranscriptDto {
    fn from(t: PodcastEpisodeTranscript) -> Self {
        Self {
            id: t.id.to_string(),
            source: t.source.as_str().to_string(),
            language: t.language,
            mime_type: t.mime_type,
            status: t.status.as_str().to_string(),
            error: t.error,
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptSegmentDto {
    pub idx: i32,
    pub start_ms: Option<i32>,
    pub end_ms: Option<i32>,
    pub speaker: Option<String>,
    pub text: String,
}

impl From<TranscriptSegment> for TranscriptSegmentDto {
    fn from(s: TranscriptSegment) -> Self {
        Self {
            idx: s.idx,
            start_ms: s.start_ms,
            end_ms: s.end_ms,
            speaker: s.speaker,
            text: s.text,
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptWithSegmentsDto {
    pub id: String,
    pub source: String,
    pub language: Option<String>,
    pub mime_type: String,
    pub status: String,
    pub error: Option<String>,
    pub segments: Vec<TranscriptSegmentDto>,
}

impl TranscriptWithSegmentsDto {
    fn new(transcript: PodcastEpisodeTranscript, segments: Vec<TranscriptSegment>) -> Self {
        Self {
            id: transcript.id.to_string(),
            source: transcript.source.as_str().to_string(),
            language: transcript.language,
            mime_type: transcript.mime_type,
            status: transcript.status.as_str().to_string(),
            error: transcript.error,
            segments: segments.into_iter().map(TranscriptSegmentDto::from).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptSearchHitDto {
    pub transcript_id: String,
    pub start_ms: Option<i32>,
    pub snippet: String,
    pub rank: f32,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptSearchGroupDto {
    pub episode_id: String,
    pub hits: Vec<TranscriptSearchHitDto>,
}

impl From<crate::services::transcript::service::TranscriptSearchGroup> for TranscriptSearchGroupDto {
    fn from(group: crate::services::transcript::service::TranscriptSearchGroup) -> Self {
        Self {
            episode_id: group.episode_id.to_string(),
            hits: group
                .hits
                .into_iter()
                .map(|h| TranscriptSearchHitDto {
                    transcript_id: h.transcript_id.to_string(),
                    start_ms: h.start_ms,
                    snippet: h.snippet,
                    rank: h.rank,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReparseReportDto {
    pub reparsed: usize,
    pub failed: usize,
}

impl From<crate::services::transcript::service::ReparseReport> for ReparseReportDto {
    fn from(r: crate::services::transcript::service::ReparseReport) -> Self {
        Self {
            reparsed: r.reparsed,
            failed: r.failed,
        }
    }
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptSearchQuery {
    pub q: String,
    pub podcast_id: Option<String>,
    pub page: Option<i64>,
}

// ── helpers ───────────────────────────────────────────────────────────────

/// Looks up a transcript by id and checks it actually belongs to `episode_id`
/// and has an archived file — collapses "wrong episode", "unknown transcript
/// id" and "not archived yet" all into a plain 404 so callers can't probe for
/// the existence of transcripts on episodes they don't have access to.
fn find_archived_transcript_for_episode(
    state: &AppState,
    episode_id: Uuid,
    transcript_id: Uuid,
) -> Result<PodcastEpisodeTranscript, CustomError> {
    let transcript = state
        .transcript_service
        .get_by_id(transcript_id)?
        .filter(|t| t.episode_id == episode_id)
        .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(Warning)))?;

    if transcript.file_path.is_none() {
        return Err(CustomErrorInner::NotFound(Warning).into());
    }

    Ok(transcript)
}

async fn stream_transcript_file(transcript: PodcastEpisodeTranscript) -> Result<Response, CustomError> {
    let file_path = transcript
        .file_path
        .as_deref()
        .expect("caller only passes transcripts with a file_path");

    let bytes = tokio::fs::read(file_path).await.map_err(|err| {
        tracing::error!("could not read archived transcript {file_path}: {err}");
        CustomError::from(CustomErrorInner::NotFound(Warning))
    })?;

    let mut headers = http::HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&transcript.mime_type).unwrap_or(HeaderValue::from_static("application/octet-stream")),
    );

    Ok((StatusCode::OK, headers, Body::from(bytes)).into_response())
}

// ── handlers ──────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/podcasts/episodes/{id}/transcripts",
    responses(
        (status = 200, description = "Lists every transcript known for the episode.", body = [TranscriptDto])
    ),
    tag = "transcripts"
)]
pub async fn get_transcripts_of_episode(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Extension(_requester): Extension<User>,
) -> Result<Json<Vec<TranscriptDto>>, CustomError> {
    let episode_id = resolve_episode_uuid(&id)?;
    let transcripts = state
        .transcript_service
        .get_by_episode_id(episode_id)?
        .into_iter()
        .map(TranscriptDto::from)
        .collect();
    Ok(Json(transcripts))
}

#[utoipa::path(
    get,
    path = "/podcasts/episodes/{id}/transcript",
    responses(
        (status = 200, description = "The episode's preferred transcript with its segments.", body = TranscriptWithSegmentsDto),
        (status = 404, description = "The episode has no preferred (parsed) transcript yet.")
    ),
    tag = "transcripts"
)]
pub async fn get_preferred_transcript(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Extension(_requester): Extension<User>,
) -> Result<Json<TranscriptWithSegmentsDto>, CustomError> {
    let episode_id = resolve_episode_uuid(&id)?;
    let preferred = state.transcript_service.get_preferred_segments(episode_id)?;

    match preferred {
        Some((transcript, segments)) => Ok(Json(TranscriptWithSegmentsDto::new(transcript, segments))),
        None => Err(CustomErrorInner::NotFound(Warning).into()),
    }
}

#[utoipa::path(
    get,
    path = "/podcasts/episodes/{id}/transcripts/{tid}/file",
    responses(
        (status = 200, description = "Streams the transcript's archived file with its stored mime type."),
        (status = 404, description = "Transcript not found, not archived yet, or belongs to a different episode.")
    ),
    tag = "transcripts"
)]
pub async fn get_transcript_file(
    State(state): State<AppState>,
    Path((id, tid)): Path<(String, String)>,
    Extension(_requester): Extension<User>,
) -> Result<Response, CustomError> {
    let episode_id = resolve_episode_uuid(&id)?;
    let transcript_id = parse_transcript_uuid(&tid)?;
    let transcript = find_archived_transcript_for_episode(&state, episode_id, transcript_id)?;
    stream_transcript_file(transcript).await
}

#[utoipa::path(
    get,
    path = "/podcasts/episodes/{id}/transcripts/{tid}/file/apiKey/{api_key}",
    responses(
        (status = 200, description = "Streams the transcript's archived file with its stored mime type (apiKey auth)."),
        (status = 403, description = "Invalid apiKey."),
        (status = 404, description = "Transcript not found, not archived yet, or belongs to a different episode.")
    ),
    tag = "transcripts"
)]
pub async fn get_transcript_file_with_api_key(
    State(state): State<AppState>,
    Path((id, tid, api_key)): Path<(String, String, String)>,
) -> Result<Response, CustomError> {
    if !state.user_auth_service.is_api_key_valid(&api_key) {
        return Err(CustomErrorInner::Forbidden(Warning).into());
    }

    let episode_id = resolve_episode_uuid(&id)?;
    let transcript_id = parse_transcript_uuid(&tid)?;
    let transcript = find_archived_transcript_for_episode(&state, episode_id, transcript_id)?;
    stream_transcript_file(transcript).await
}

#[utoipa::path(
    post,
    path = "/podcasts/episodes/{id}/transcribe",
    responses(
        (status = 200, description = "A generated-transcript job was enqueued for the episode."),
        (status = 409, description = "A transcription job already exists for this episode."),
        (status = 503, description = "No transcription backend is configured.")
    ),
    tag = "transcripts"
)]
pub async fn enqueue_transcription(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Extension(requester): Extension<User>,
) -> Result<Response, CustomError> {
    if !requester.is_privileged_user() {
        return Err(CustomErrorInner::Forbidden(Warning).into());
    }

    if ENVIRONMENT_SERVICE.transcription_config.is_none() {
        return Ok((StatusCode::SERVICE_UNAVAILABLE, "no transcription backend is configured").into_response());
    }

    let episode_id = resolve_episode_uuid(&id)?;
    match state.transcript_service.enqueue_job(episode_id)? {
        Some(_job) => Ok(StatusCode::OK.into_response()),
        None => Err(CustomErrorInner::Conflict(
            "a transcription job already exists for this episode".to_string(),
            Warning,
        )
        .into()),
    }
}

#[utoipa::path(
    get,
    path = "/transcripts/search",
    params(TranscriptSearchQuery),
    responses(
        (status = 200, description = "Transcript segments matching the query, grouped by episode.", body = [TranscriptSearchGroupDto])
    ),
    tag = "transcripts"
)]
pub async fn search_transcripts(
    State(state): State<AppState>,
    Query(params): Query<TranscriptSearchQuery>,
    Extension(_requester): Extension<User>,
) -> Result<Json<Vec<TranscriptSearchGroupDto>>, CustomError> {
    // `page` is 0-based; the repository itself does not validate it, so a
    // negative value is clamped here rather than passed through untouched.
    let page = params.page.unwrap_or(0).max(0);
    let podcast_id = params
        .podcast_id
        .as_deref()
        .map(|s| {
            Uuid::parse_str(s)
                .map_err(|_| CustomError::from(CustomErrorInner::BadRequest("invalid podcastId".to_string(), Warning)))
        })
        .transpose()?;

    let groups = state
        .transcript_service
        .search(params.q.trim(), podcast_id, page)?
        .into_iter()
        .map(TranscriptSearchGroupDto::from)
        .collect();

    Ok(Json(groups))
}

#[utoipa::path(
    post,
    path = "/settings/transcripts/reparse",
    responses(
        (status = 200, description = "Re-parsed every archived transcript.", body = ReparseReportDto)
    ),
    tag = "transcripts"
)]
pub async fn reparse_transcripts(
    State(state): State<AppState>,
    Extension(requester): Extension<User>,
) -> Result<Json<ReparseReportDto>, CustomError> {
    if !requester.is_admin() {
        return Err(CustomErrorInner::Forbidden(Warning).into());
    }

    let report = state.transcript_service.reparse_all()?;
    Ok(Json(ReparseReportDto::from(report)))
}

fn parse_transcript_uuid(id: &str) -> Result<Uuid, CustomError> {
    Uuid::parse_str(id).map_err(|_| CustomErrorInner::BadRequest("invalid transcript id".to_string(), Warning).into())
}

pub fn get_transcript_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_transcripts_of_episode))
        .routes(routes!(get_preferred_transcript))
        .routes(routes!(get_transcript_file))
        .routes(routes!(get_transcript_file_with_api_key))
        .routes(routes!(enqueue_transcription))
        .routes(routes!(search_transcripts))
        .routes(routes!(reparse_transcripts))
}

#[cfg(all(test, feature = "sqlite"))]
mod tests {
    use crate::test_support::tests::handle_test_startup;
    use crate::test_utils::test_builder::user_test_builder::tests::UserTestDataBuilder;
    use axum::Extension;
    use axum::extract::{Path, State};
    use common_infrastructure::error::CustomErrorInner;
    use diesel::prelude::*;
    use podfetch_domain::podcast_episode_transcript::{
        PodcastEpisodeTranscriptRepository, TranscriptSegment, TranscriptSource, TranscriptStatus, UpsertTranscript,
    };
    use podfetch_domain::user::User;
    use podfetch_persistence::adapters::PodcastEpisodeTranscriptRepositoryImpl;
    use podfetch_persistence::db::{database, get_connection};
    use podfetch_persistence::schema::podcast_episodes::dsl as pe_dsl;
    use serde_json::{Value, json};
    use serial_test::serial;
    use uuid::Uuid;

    fn unique(prefix: &str) -> String {
        format!("{prefix}-{}", Uuid::new_v4())
    }

    fn non_admin_user() -> User {
        UserTestDataBuilder::new().build()
    }

    fn app_state() -> crate::app_state::AppState {
        crate::app_state::AppState::new()
    }

    /// Creates a podcast + episode pair and returns the episode's uuid.
    fn seed_episode() -> Uuid {
        let slug = unique("transcript-ctrl-podcast");
        let podcast = crate::services::podcast::service::PodcastService::add_podcast_to_database(
            &slug,
            &slug,
            &format!("https://example.com/{slug}.xml"),
            "http://localhost:8080/ui/default.jpg",
            &slug,
        )
        .unwrap();

        let episode_id = Uuid::new_v4();
        let episode_id_str = episode_id.to_string();
        diesel::insert_into(pe_dsl::podcast_episodes)
            .values((
                pe_dsl::id.eq(episode_id_str.clone()),
                pe_dsl::podcast_id.eq(podcast.id.clone()),
                pe_dsl::episode_id.eq(unique("episode")),
                pe_dsl::name.eq("Transcript Controller Test Episode".to_string()),
                pe_dsl::url.eq(format!("https://example.com/{episode_id_str}.mp3")),
                pe_dsl::date_of_recording.eq("2026-03-01T00:00:00Z".to_string()),
                pe_dsl::image_url.eq("http://localhost:8080/ui/default.jpg".to_string()),
                pe_dsl::total_time.eq(1800),
                pe_dsl::description.eq("transcript controller test".to_string()),
                pe_dsl::guid.eq(unique("guid")),
                pe_dsl::deleted.eq(false),
                pe_dsl::episode_numbering_processed.eq(false),
            ))
            .execute(&mut get_connection())
            .unwrap();

        episode_id
    }

    /// Seeds one `parsed` + preferred transcript with segments and an
    /// archived file on disk, returning (transcript_id, archive_path).
    fn seed_parsed_transcript_with_file(episode_id: Uuid, text: &str) -> (Uuid, std::path::PathBuf) {
        let repo = PodcastEpisodeTranscriptRepositoryImpl::new(database());
        let transcript_id = repo
            .upsert(UpsertTranscript {
                episode_id,
                source: TranscriptSource::Feed,
                original_url: Some(format!("https://example.com/{episode_id}.vtt")),
                mime_type: "text/vtt".to_string(),
                language: Some("en".to_string()),
            })
            .unwrap();

        let segments = vec![TranscriptSegment {
            idx: 0,
            start_ms: Some(500),
            end_ms: Some(4200),
            speaker: None,
            text: text.to_string(),
        }];
        repo.replace_segments(transcript_id, &segments).unwrap();
        repo.set_status(transcript_id, TranscriptStatus::Parsed, None).unwrap();
        repo.set_preferred(episode_id, Some(transcript_id)).unwrap();

        let archive_path = std::env::temp_dir().join(format!("transcript-ctrl-{transcript_id}.vtt"));
        std::fs::write(&archive_path, "WEBVTT\n\n00:00:00.500 --> 00:00:04.200\nHello world\n").unwrap();
        repo.set_file_path(transcript_id, archive_path.to_str().unwrap()).unwrap();

        (transcript_id, archive_path)
    }

    // ── GET /transcripts + GET /transcript (no data) ────────────────────

    #[tokio::test]
    #[serial]
    async fn episode_without_transcripts_returns_empty_list_and_404_for_preferred() {
        let server = handle_test_startup().await;
        let episode_id = seed_episode();

        let list_response = server
            .test_server
            .get(&format!("/api/v1/podcasts/episodes/{episode_id}/transcripts"))
            .await;
        assert_eq!(list_response.status_code(), 200);
        assert!(list_response.json::<Value>().as_array().unwrap().is_empty());

        let preferred_response = server
            .test_server
            .get(&format!("/api/v1/podcasts/episodes/{episode_id}/transcript"))
            .await;
        assert_eq!(preferred_response.status_code(), 404);
    }

    // ── GET /transcripts + GET /transcript (seeded) ─────────────────────

    #[tokio::test]
    #[serial]
    async fn episode_with_seeded_transcript_returns_list_and_preferred_content() {
        let server = handle_test_startup().await;
        let episode_id = seed_episode();
        let (transcript_id, archive_path) = seed_parsed_transcript_with_file(episode_id, "hello world unique-marker");

        let list_response = server
            .test_server
            .get(&format!("/api/v1/podcasts/episodes/{episode_id}/transcripts"))
            .await;
        assert_eq!(list_response.status_code(), 200);
        let list = list_response.json::<Value>();
        let list = list.as_array().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0]["id"], json!(transcript_id.to_string()));
        assert_eq!(list[0]["status"], json!("parsed"));
        assert_eq!(list[0]["mimeType"], json!("text/vtt"));

        let preferred_response = server
            .test_server
            .get(&format!("/api/v1/podcasts/episodes/{episode_id}/transcript"))
            .await;
        assert_eq!(preferred_response.status_code(), 200);
        let preferred = preferred_response.json::<Value>();
        assert_eq!(preferred["id"], json!(transcript_id.to_string()));
        let segments = preferred["segments"].as_array().unwrap();
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0]["text"], json!("hello world unique-marker"));

        let _ = std::fs::remove_file(&archive_path);
    }

    // ── GET .../file + .../file/apiKey/{key} ────────────────────────────

    #[tokio::test]
    #[serial]
    async fn get_transcript_file_streams_bytes_with_stored_mime_type() {
        let server = handle_test_startup().await;
        let episode_id = seed_episode();
        let (transcript_id, archive_path) = seed_parsed_transcript_with_file(episode_id, "file streaming test");

        let response = server
            .test_server
            .get(&format!(
                "/api/v1/podcasts/episodes/{episode_id}/transcripts/{transcript_id}/file"
            ))
            .await;
        assert_eq!(response.status_code(), 200);
        assert_eq!(
            response.headers().get("content-type").unwrap().to_str().unwrap(),
            "text/vtt"
        );
        assert!(response.text().contains("Hello world"));

        let _ = std::fs::remove_file(&archive_path);
    }

    #[tokio::test]
    #[serial]
    async fn get_transcript_file_returns_404_for_wrong_episode() {
        let server = handle_test_startup().await;
        let episode_id = seed_episode();
        let other_episode_id = seed_episode();
        let (transcript_id, archive_path) = seed_parsed_transcript_with_file(episode_id, "belongs to first episode");

        let response = server
            .test_server
            .get(&format!(
                "/api/v1/podcasts/episodes/{other_episode_id}/transcripts/{transcript_id}/file"
            ))
            .await;
        assert_eq!(response.status_code(), 404);

        let _ = std::fs::remove_file(&archive_path);
    }

    #[tokio::test]
    #[serial]
    async fn get_transcript_file_with_valid_api_key_streams_bytes() {
        let server = handle_test_startup().await;
        let episode_id = seed_episode();
        let (transcript_id, archive_path) = seed_parsed_transcript_with_file(episode_id, "api key streaming test");

        let response = server
            .test_server
            .get(&format!(
                "/api/v1/podcasts/episodes/{episode_id}/transcripts/{transcript_id}/file/apiKey/test-api-key"
            ))
            .await;
        assert_eq!(response.status_code(), 200);
        assert_eq!(
            response.headers().get("content-type").unwrap().to_str().unwrap(),
            "text/vtt"
        );

        let _ = std::fs::remove_file(&archive_path);
    }

    #[tokio::test]
    #[serial]
    async fn get_transcript_file_with_invalid_api_key_is_forbidden() {
        let server = handle_test_startup().await;
        let episode_id = seed_episode();
        let (transcript_id, archive_path) = seed_parsed_transcript_with_file(episode_id, "invalid api key test");

        let response = server
            .test_server
            .get(&format!(
                "/api/v1/podcasts/episodes/{episode_id}/transcripts/{transcript_id}/file/apiKey/not-a-real-key"
            ))
            .await;
        assert_eq!(response.status_code(), 403);

        let _ = std::fs::remove_file(&archive_path);
    }

    // ── GET /transcripts/search ──────────────────────────────────────────

    #[tokio::test]
    #[serial]
    async fn search_finds_seeded_segment() {
        let server = handle_test_startup().await;
        let episode_id = seed_episode();
        let (_transcript_id, archive_path) =
            seed_parsed_transcript_with_file(episode_id, "a very distinctive zzyzx search term appears here");

        let response = server
            .test_server
            .get("/api/v1/transcripts/search?q=zzyzx&page=0")
            .await;
        assert_eq!(response.status_code(), 200);
        let groups = response.json::<Value>();
        let groups = groups.as_array().unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0]["episodeId"], json!(episode_id.to_string()));
        let hits = groups[0]["hits"].as_array().unwrap();
        assert_eq!(hits.len(), 1);
        assert!(hits[0]["snippet"].as_str().unwrap().contains("zzyzx"));

        let _ = std::fs::remove_file(&archive_path);
    }

    #[tokio::test]
    #[serial]
    async fn search_with_no_match_returns_empty_list() {
        let server = handle_test_startup().await;

        let response = server
            .test_server
            .get("/api/v1/transcripts/search?q=no-such-term-anywhere&page=0")
            .await;
        assert_eq!(response.status_code(), 200);
        assert!(response.json::<Value>().as_array().unwrap().is_empty());
    }

    // ── POST /podcasts/episodes/{id}/transcribe ─────────────────────────

    #[tokio::test]
    #[serial]
    async fn enqueue_transcription_without_config_returns_503() {
        let server = handle_test_startup().await;
        let episode_id = seed_episode();

        let response = server
            .test_server
            .post(&format!("/api/v1/podcasts/episodes/{episode_id}/transcribe"))
            .await;
        assert_eq!(response.status_code(), 503);
    }

    #[tokio::test]
    #[serial]
    async fn enqueue_transcription_rejects_non_privileged_user() {
        let non_privileged = non_admin_user();

        let result = super::enqueue_transcription(
            State(app_state()),
            Path(Uuid::new_v4().to_string()),
            Extension(non_privileged),
        )
        .await;

        match result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden for a non-privileged user"),
        }
    }

    // ── POST /settings/transcripts/reparse ───────────────────────────────

    #[tokio::test]
    #[serial]
    async fn reparse_transcripts_forbidden_for_non_admin() {
        let server = handle_test_startup().await;
        let non_admin = non_admin_user();

        // Exercised directly (not over HTTP): the test server always
        // authenticates as the configured admin user, so a non-admin caller
        // is simulated by invoking the handler with a non-admin `Extension`,
        // matching the pattern used by other controllers' forbidden tests.
        let _server = server; // keep the DB/mutex guard alive for the duration
        let result = super::reparse_transcripts(State(app_state()), Extension(non_admin)).await;

        match result {
            Err(err) => assert!(matches!(err.inner, CustomErrorInner::Forbidden(_))),
            Ok(_) => panic!("expected forbidden for a non-admin user"),
        }
    }

    #[tokio::test]
    #[serial]
    async fn reparse_transcripts_succeeds_for_admin_over_http() {
        let server = handle_test_startup().await;

        let response = server.test_server.post("/api/v1/settings/transcripts/reparse").await;
        assert_eq!(response.status_code(), 200);
        let body = response.json::<Value>();
        assert_eq!(body["reparsed"], json!(0));
        assert_eq!(body["failed"], json!(0));
    }
}
