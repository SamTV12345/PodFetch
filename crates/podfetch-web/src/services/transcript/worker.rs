//! Background job worker for Whisper-generated transcripts.
//!
//! [`process_one_job`] is the synchronous, testable core: claim the oldest
//! pending [`TranscriptionJob`], transcribe its episode's local audio file
//! through a [`WhisperClient`], and persist the result via
//! [`TranscriptService::store_generated`]. [`run_transcription_worker`] is the
//! thin async loop that drives it forever, always inside
//! `tokio::task::spawn_blocking` since `WhisperClient::transcribe` is a
//! blocking HTTP call that can legitimately run for minutes.

use crate::server::ChatServerHandle;
use crate::services::transcript::service::TranscriptService;
use crate::services::transcript::whisper_client::WhisperClient;
use crate::usecases::podcast_episode::PodcastEpisodeUseCase;
use common_infrastructure::error::{CustomError, CustomErrorInner, ErrorSeverity};
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
use podfetch_domain::podcast_episode_transcript::{
    TranscriptionJob, TranscriptionJobRepository, TranscriptionJobStatus,
};
use podfetch_persistence::adapters::TranscriptionJobRepositoryImpl;
use podfetch_persistence::db::database;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

/// After this many failed attempts a job is given up on (`failed`) instead of
/// being put back on the queue (`pending`).
const MAX_ATTEMPTS: i32 = 3;
/// How long the async loop sleeps after finding no pending job.
const POLL_INTERVAL: Duration = Duration::from_secs(15);

/// Processes at most one pending transcription job.
///
/// Returns `Ok(false)` when the queue is empty (the caller decides whether to
/// sleep). Returns `Ok(true)` whenever a job was claimed, regardless of
/// whether transcription itself succeeded — a Whisper/IO failure is recorded
/// on the job row (`pending` with an `error` while `attempts < 3`, `failed`
/// once `attempts` reaches 3) rather than bubbled up, mirroring how
/// `TranscriptService::process_pending_for_episode` never lets one broken
/// transcript abort the rest of the queue. Only a genuine repository error
/// while claiming/updating the job itself propagates as `Err`.
fn process_one_job(
    job_repo: &dyn TranscriptionJobRepository<Error = CustomError>,
    service: &TranscriptService,
    client: &WhisperClient,
) -> Result<bool, CustomError> {
    let Some(job) = job_repo.next_pending()? else {
        return Ok(false);
    };
    let episode_id = job.episode_id.to_string();

    job_repo.set_status(job.id, TranscriptionJobStatus::Running, None)?;
    ChatServerHandle::broadcast_transcription_status(&episode_id, TranscriptionJobStatus::Running.as_str(), None);

    if let Err(err) = transcribe_job(&job, service, client) {
        let error_message = err.to_string();
        let attempts = job_repo.increment_attempts(job.id)?;
        let (status, error_for_broadcast) = if attempts >= MAX_ATTEMPTS {
            (TranscriptionJobStatus::Failed, Some(error_message.as_str()))
        } else {
            (TranscriptionJobStatus::Pending, Some(error_message.as_str()))
        };
        job_repo.set_status(job.id, status.clone(), error_for_broadcast)?;
        ChatServerHandle::broadcast_transcription_status(&episode_id, status.as_str(), error_for_broadcast);
        return Ok(true);
    }

    job_repo.set_status(job.id, TranscriptionJobStatus::Done, None)?;
    ChatServerHandle::broadcast_transcription_status(&episode_id, TranscriptionJobStatus::Done.as_str(), None);
    Ok(true)
}

/// Loads the job's episode, sends its local audio file to Whisper, and
/// stores the resulting segments as the episode's generated transcript.
fn transcribe_job(job: &TranscriptionJob, service: &TranscriptService, client: &WhisperClient) -> Result<(), CustomError> {
    let episode = PodcastEpisodeUseCase::get_podcast_episode_by_internal_id(job.episode_id)?
        .ok_or_else(|| CustomError::from(CustomErrorInner::NotFound(ErrorSeverity::Warning)))?;

    let audio_path = episode.file_episode_path.as_deref().ok_or_else(|| {
        CustomError::from(CustomErrorInner::Conflict(
            "cannot transcribe: episode has no downloaded audio file".to_string(),
            ErrorSeverity::Warning,
        ))
    })?;

    let (segments, language) = client.transcribe(Path::new(audio_path))?;
    service.store_generated(&episode, segments, language)
}

/// Endless background loop that drains the transcription job queue one job
/// at a time. A no-op when no transcription backend is configured — safe to
/// call unconditionally at startup, the check happens inside.
///
/// Resets any job left `running` from a previous, uncleanly-stopped process
/// back to `pending` once, up front, before entering the loop.
pub async fn run_transcription_worker() {
    let Some(config) = ENVIRONMENT_SERVICE.transcription_config.clone() else {
        tracing::info!("Transcription worker not starting: no transcription backend configured");
        return;
    };
    run_worker_with_config(config, Arc::new(std::sync::atomic::AtomicBool::new(false))).await
}

/// The actual worker: one dedicated blocking thread runs the whole
/// claim-transcribe-record loop until `stop` is set (only tests ever set it).
///
/// [`WhisperClient`] wraps `reqwest::blocking::Client`, which spins up (and on
/// drop tears down) its own internal tokio runtime — touching it from an async
/// worker thread panics with "Cannot drop a runtime in a context where
/// blocking is not allowed" and silently kills the worker task. The client is
/// therefore constructed, used and dropped exclusively inside this single
/// `spawn_blocking` closure and never crosses into async context.
async fn run_worker_with_config(
    config: common_infrastructure::config::TranscriptionConfig,
    stop: Arc<std::sync::atomic::AtomicBool>,
) {
    use std::sync::atomic::Ordering;

    let job_repo: Arc<TranscriptionJobRepositoryImpl> = Arc::new(TranscriptionJobRepositoryImpl::new(database()));
    match job_repo.reset_running_to_pending() {
        Ok(0) => {}
        Ok(reset) => {
            tracing::info!("Reset {reset} stuck transcription job(s) from running back to pending at startup")
        }
        Err(err) => tracing::error!("Failed to reset stuck transcription jobs at startup: {err}"),
    }

    let service = Arc::new(TranscriptService::default_service());

    let outcome = tokio::task::spawn_blocking(move || {
        // Sleeps in small slices so a stop request never waits a full poll interval.
        let sleep_unless_stopped = |stop: &std::sync::atomic::AtomicBool| {
            let slice = Duration::from_millis(200);
            let mut slept = Duration::ZERO;
            while slept < POLL_INTERVAL && !stop.load(Ordering::Relaxed) {
                std::thread::sleep(slice);
                slept += slice;
            }
        };

        let client = WhisperClient::new(config);
        while !stop.load(Ordering::Relaxed) {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                process_one_job(job_repo.as_ref(), service.as_ref(), &client)
            }));
            match result {
                Ok(Ok(true)) => {}
                Ok(Ok(false)) => sleep_unless_stopped(&stop),
                Ok(Err(err)) => {
                    tracing::error!("Transcription worker: job processing failed: {err}");
                    sleep_unless_stopped(&stop);
                }
                Err(_) => {
                    tracing::error!("Transcription worker: job processing panicked");
                    sleep_unless_stopped(&stop);
                }
            }
        }
    })
    .await;

    if let Err(join_err) = outcome {
        tracing::error!("Transcription worker stopped unexpectedly: {join_err}");
    }
}

#[cfg(all(test, feature = "sqlite"))]
mod tests {
    use super::*;
    use crate::test_support::tests::{GLOBAL_MUTEX, ensure_test_env_vars};
    use axum::Router;
    use axum::routing::post;
    use common_infrastructure::config::TranscriptionConfig;
    use diesel::prelude::*;
    use podfetch_domain::podcast_episode_transcript::{
        PodcastEpisodeTranscriptRepository, TranscriptSource, TranscriptStatus,
    };
    use podfetch_persistence::db::{get_connection, run_migrations};
    use podfetch_persistence::schema::{podcast_episodes, podcasts};
    use podfetch_persistence::podcast_episode_transcript::DieselPodcastEpisodeTranscriptRepository;
    use std::sync::MutexGuard;
    use uuid::Uuid;

    /// Regression test for the startup panic that silently killed the worker:
    /// `reqwest::blocking::Client` must never touch the async runtime (see
    /// `run_worker_with_config`). The worker task must still be alive and
    /// polling after startup instead of having died on a construction panic.
    #[tokio::test]
    async fn worker_survives_startup_inside_the_async_runtime() {
        let _guard = lock_and_prepare_db();
        let config = TranscriptionConfig {
            base_url: "http://127.0.0.1:1".to_string(),
            api_key: None,
            model: "whisper-1".to_string(),
        };

        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let handle = tokio::spawn(run_worker_with_config(config, stop.clone()));
        tokio::time::sleep(std::time::Duration::from_millis(400)).await;
        assert!(
            !handle.is_finished(),
            "worker task ended right after startup — it should be alive and polling"
        );
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        handle.await.unwrap();
    }

    fn lock_and_prepare_db() -> MutexGuard<'static, ()> {
        ensure_test_env_vars();
        let guard = GLOBAL_MUTEX.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        run_migrations();

        let mut conn = get_connection();
        for table in [
            "podcast_episode_transcript_segments",
            "podcast_episode_transcripts",
            "transcription_jobs",
            "podcast_episodes",
            "podcasts",
        ] {
            let _ = diesel::sql_query(format!("DELETE FROM {table}")).execute(&mut conn);
        }

        guard
    }

    #[derive(Insertable)]
    #[diesel(table_name = podcasts)]
    struct SeedPodcast {
        id: String,
        name: String,
        directory_id: String,
        rssfeed: String,
        image_url: String,
        active: bool,
        original_image_url: String,
        directory_name: String,
    }

    #[derive(Insertable)]
    #[diesel(table_name = podcast_episodes)]
    struct SeedEpisode {
        id: String,
        podcast_id: String,
        episode_id: String,
        name: String,
        url: String,
        date_of_recording: String,
        image_url: String,
        total_time: i32,
        description: String,
        guid: String,
        deleted: bool,
        episode_numbering_processed: bool,
        file_episode_path: Option<String>,
    }

    fn seed_podcast() -> Uuid {
        let id = Uuid::new_v4();
        diesel::insert_into(podcasts::table)
            .values(SeedPodcast {
                id: id.to_string(),
                name: format!("Test Podcast {id}"),
                directory_id: Uuid::new_v4().to_string(),
                rssfeed: format!("https://example.com/feed/{id}.xml"),
                image_url: "https://example.com/img.png".to_string(),
                active: true,
                original_image_url: "https://example.com/img.png".to_string(),
                directory_name: format!("podcast-{id}"),
            })
            .execute(&mut get_connection())
            .expect("seed podcast");
        id
    }

    fn seed_episode(podcast_id: Uuid, file_episode_path: Option<&str>) -> Uuid {
        let id = Uuid::new_v4();
        diesel::insert_into(podcast_episodes::table)
            .values(SeedEpisode {
                id: id.to_string(),
                podcast_id: podcast_id.to_string(),
                episode_id: Uuid::new_v4().to_string(),
                name: "Test Episode".to_string(),
                url: format!("https://example.com/ep/{id}.mp3"),
                date_of_recording: "2024-01-01".to_string(),
                image_url: "https://example.com/ep.png".to_string(),
                total_time: 3600,
                description: "Test description".to_string(),
                guid: Uuid::new_v4().to_string(),
                deleted: false,
                episode_numbering_processed: false,
                file_episode_path: file_episode_path.map(|s| s.to_string()),
            })
            .execute(&mut get_connection())
            .expect("seed episode");
        id
    }

    fn temp_episode_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("podfetch-worker-test-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create temp episode dir");
        dir
    }

    fn transcript_service() -> TranscriptService {
        TranscriptService::default_service()
    }

    fn job_repo() -> TranscriptionJobRepositoryImpl {
        TranscriptionJobRepositoryImpl::new(database())
    }

    fn whisper_config(base_url: String) -> TranscriptionConfig {
        TranscriptionConfig {
            base_url,
            api_key: None,
            model: "whisper-1".to_string(),
        }
    }

    fn spawn_mock_server(app: Router) -> String {
        let (addr_tx, addr_rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("build mock server runtime");
            rt.block_on(async move {
                let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                    .await
                    .expect("bind mock whisper server");
                addr_tx.send(listener.local_addr().unwrap()).unwrap();
                axum::serve(listener, app).await.unwrap();
            });
        });
        let addr = addr_rx.recv().expect("mock server address");
        format!("http://{addr}")
    }

    const VERBOSE_JSON_OK: &str = r#"{
        "language": "english",
        "segments": [
            {"start": 0.0, "end": 1.5, "text": "Hello from whisper"}
        ]
    }"#;

    // ── (a) no job -> Ok(false) ──────────────────────────────────────────

    #[test]
    fn process_one_job_returns_false_when_queue_is_empty() {
        let _guard = lock_and_prepare_db();
        let repo = job_repo();
        let service = transcript_service();
        let client = WhisperClient::new(whisper_config("http://127.0.0.1:0".to_string()));

        let found = process_one_job(&repo, &service, &client).expect("must not error on empty queue");
        assert!(!found, "no pending job must yield Ok(false)");
    }

    // ── (b) job + mock whisper server ok -> done + parsed generated transcript ──

    #[test]
    fn process_one_job_transcribes_successfully_and_marks_job_done() {
        let _guard = lock_and_prepare_db();
        let repo = job_repo();
        let service = transcript_service();

        let podcast_id = seed_podcast();
        let dir = temp_episode_dir();
        let audio_path = dir.join("episode.mp3");
        std::fs::write(&audio_path, b"fake-audio").unwrap();
        let episode_id = seed_episode(podcast_id, Some(audio_path.to_str().unwrap()));

        let app = Router::new().route(
            "/v1/audio/transcriptions",
            post(|| async { ([("content-type", "application/json")], VERBOSE_JSON_OK) }),
        );
        let base_url = spawn_mock_server(app);
        let client = WhisperClient::new(whisper_config(base_url));

        let job = repo.enqueue(episode_id).expect("enqueue").expect("job created");

        let found = process_one_job(&repo, &service, &client).expect("process_one_job");
        assert!(found, "a pending job must be picked up");

        let updated_job = repo
            .get_by_episode_id(episode_id)
            .expect("get_by_episode_id")
            .expect("job row still exists");
        assert_eq!(updated_job.id, job.id);
        assert_eq!(updated_job.status, TranscriptionJobStatus::Done);
        assert_eq!(updated_job.attempts, 0, "a successful attempt must not increment attempts");

        let transcript_repo = DieselPodcastEpisodeTranscriptRepository::new(database());
        let transcripts = transcript_repo.get_by_episode_id(episode_id).expect("get transcripts");
        let generated = transcripts
            .iter()
            .find(|t| t.source == TranscriptSource::Generated)
            .expect("a generated transcript row must exist");
        assert_eq!(generated.status, TranscriptStatus::Parsed);
        assert!(generated.is_preferred, "the only parsed transcript must become preferred");

        let expected_vtt_path = dir.join("episode.transcript.generated.vtt");
        assert_eq!(generated.file_path.as_deref(), expected_vtt_path.to_str());
        assert!(expected_vtt_path.exists(), "the generated VTT file must be archived on disk");
        let vtt_contents = std::fs::read_to_string(&expected_vtt_path).unwrap();
        assert!(vtt_contents.contains("Hello from whisper"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    // ── (c) whisper error: attempts < 3 -> pending, attempts == 3 -> failed ──

    #[test]
    fn process_one_job_puts_job_back_to_pending_while_attempts_below_three() {
        let _guard = lock_and_prepare_db();
        let repo = job_repo();
        let service = transcript_service();

        let podcast_id = seed_podcast();
        let dir = temp_episode_dir();
        let audio_path = dir.join("episode.mp3");
        std::fs::write(&audio_path, b"fake-audio").unwrap();
        let episode_id = seed_episode(podcast_id, Some(audio_path.to_str().unwrap()));

        let app = Router::new().route(
            "/v1/audio/transcriptions",
            post(|| async { axum::http::StatusCode::INTERNAL_SERVER_ERROR }),
        );
        let base_url = spawn_mock_server(app);
        let client = WhisperClient::new(whisper_config(base_url));

        repo.enqueue(episode_id).expect("enqueue").expect("job created");

        // First failed attempt: attempts becomes 1 (< 3) -> back to pending.
        let found = process_one_job(&repo, &service, &client).expect("process_one_job (1st attempt)");
        assert!(found);
        let after_first = repo.get_by_episode_id(episode_id).unwrap().unwrap();
        assert_eq!(after_first.attempts, 1);
        assert_eq!(after_first.status, TranscriptionJobStatus::Pending);
        assert!(after_first.error.is_some());

        // Second failed attempt: attempts becomes 2 (< 3) -> still pending.
        let found = process_one_job(&repo, &service, &client).expect("process_one_job (2nd attempt)");
        assert!(found);
        let after_second = repo.get_by_episode_id(episode_id).unwrap().unwrap();
        assert_eq!(after_second.attempts, 2);
        assert_eq!(after_second.status, TranscriptionJobStatus::Pending);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn process_one_job_marks_job_failed_once_attempts_reach_three() {
        let _guard = lock_and_prepare_db();
        let repo = job_repo();
        let service = transcript_service();

        let podcast_id = seed_podcast();
        let dir = temp_episode_dir();
        let audio_path = dir.join("episode.mp3");
        std::fs::write(&audio_path, b"fake-audio").unwrap();
        let episode_id = seed_episode(podcast_id, Some(audio_path.to_str().unwrap()));

        let app = Router::new().route(
            "/v1/audio/transcriptions",
            post(|| async { axum::http::StatusCode::INTERNAL_SERVER_ERROR }),
        );
        let base_url = spawn_mock_server(app);
        let client = WhisperClient::new(whisper_config(base_url));

        repo.enqueue(episode_id).expect("enqueue").expect("job created");

        for _ in 0..2 {
            process_one_job(&repo, &service, &client).expect("earlier attempt");
        }

        // Third failed attempt: attempts becomes 3 -> failed, with the error recorded.
        let found = process_one_job(&repo, &service, &client).expect("process_one_job (3rd attempt)");
        assert!(found);
        let after_third = repo.get_by_episode_id(episode_id).unwrap().unwrap();
        assert_eq!(after_third.attempts, 3);
        assert_eq!(after_third.status, TranscriptionJobStatus::Failed);
        assert!(after_third.error.is_some());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn process_one_job_fails_immediately_when_episode_has_no_local_audio_path() {
        let _guard = lock_and_prepare_db();
        let repo = job_repo();
        let service = transcript_service();

        let podcast_id = seed_podcast();
        let episode_id = seed_episode(podcast_id, None);
        let client = WhisperClient::new(whisper_config("http://127.0.0.1:0".to_string()));

        repo.enqueue(episode_id).expect("enqueue").expect("job created");

        let found = process_one_job(&repo, &service, &client).expect("process_one_job");
        assert!(found);
        let after = repo.get_by_episode_id(episode_id).unwrap().unwrap();
        assert_eq!(after.attempts, 1);
        assert_eq!(after.status, TranscriptionJobStatus::Pending);
        assert!(after.error.is_some());
    }
}
