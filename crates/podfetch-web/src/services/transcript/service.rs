//! Core transcript service: feed-tag bookkeeping (Flow 1), download + archive +
//! parse of pending transcripts (Flow 2), preference recomputation, search
//! grouping, and re-parsing already-archived transcripts.
//!
//! Errors that are specific to a single transcript (HTTP failure, oversized
//! body, unparsable format) are ALWAYS captured on that transcript's row
//! (`status = 'failed'`, `error = ...`) rather than bubbled up — callers like
//! `process_pending_for_episode` must keep working through the rest of an
//! episode's transcripts and other episodes even when one transcript is
//! broken. Only genuine infrastructure failures (the repository itself
//! erroring) propagate as `Err`.

use crate::services::transcript::parser::{self, TranscriptFormat};
use crate::services::transcript::whisper_client;
use common_infrastructure::error::{CustomError, CustomErrorInner, ErrorSeverity, map_reqwest_error};
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
use podfetch_domain::podcast_episode_transcript::{
    PodcastEpisodeTranscript, PodcastEpisodeTranscriptRepository, TranscriptSearchHit, TranscriptSegment,
    TranscriptSource, TranscriptStatus, TranscriptionJob, TranscriptionJobRepository, UpsertTranscript,
};
use podfetch_persistence::adapters::{PodcastEpisodeTranscriptRepositoryImpl, TranscriptionJobRepositoryImpl};
use podfetch_persistence::db::database;
use podfetch_persistence::podcast_episode::PodcastEpisodeEntity as PodcastEpisode;
use podfetch_storage::FileHandleWrapper;
use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

/// HTTP timeout for a single transcript download.
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(30);
/// Hard cap on a downloaded transcript body. Anything larger is rejected.
const MAX_TRANSCRIPT_BYTES: usize = 20 * 1024 * 1024;
/// Internal page size used when paging through the repository's flat search
/// hits before grouping/capping them per episode.
const SEARCH_INTERNAL_PAGE_SIZE: i64 = 60;
/// Max segments surfaced per episode in a single grouped search result.
const MAX_HITS_PER_EPISODE: usize = 3;

/// A `<podcast:transcript>` tag as read from an episode's RSS feed entry.
#[derive(Debug, Clone)]
pub struct FeedTranscriptTag {
    pub url: String,
    pub mime_type: String,
    pub language: Option<String>,
}

/// A group of search hits belonging to one episode, capped and ordered by
/// [`TranscriptService::search`].
#[derive(Debug, Clone)]
pub struct TranscriptSearchGroup {
    pub episode_id: Uuid,
    pub hits: Vec<TranscriptSearchHit>,
}

/// Outcome of [`TranscriptService::reparse_all`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReparseReport {
    pub reparsed: usize,
    pub failed: usize,
}

pub struct TranscriptService {
    transcript_repo: Arc<dyn PodcastEpisodeTranscriptRepository<Error = CustomError>>,
    job_repo: Arc<dyn TranscriptionJobRepository<Error = CustomError>>,
}

impl TranscriptService {
    pub fn new(
        transcript_repo: Arc<dyn PodcastEpisodeTranscriptRepository<Error = CustomError>>,
        job_repo: Arc<dyn TranscriptionJobRepository<Error = CustomError>>,
    ) -> Self {
        Self {
            transcript_repo,
            job_repo,
        }
    }

    pub fn default_service() -> Self {
        Self::new(
            Arc::new(PodcastEpisodeTranscriptRepositoryImpl::new(database())),
            Arc::new(TranscriptionJobRepositoryImpl::new(database())),
        )
    }

    /// Flow 1: upsert the transcript tags read straight out of the episode's
    /// feed entry. Pure DB bookkeeping — no HTTP, no parsing. The rows land
    /// as `status = 'pending'` and are picked up later by
    /// [`Self::process_pending_for_episode`].
    pub fn upsert_from_feed(&self, episode_id: Uuid, tags: &[FeedTranscriptTag]) -> Result<(), CustomError> {
        for tag in tags {
            self.transcript_repo.upsert(UpsertTranscript {
                episode_id,
                source: TranscriptSource::Feed,
                original_url: Some(tag.url.clone()),
                mime_type: tag.mime_type.clone(),
                language: tag.language.clone(),
            })?;
        }
        Ok(())
    }

    /// Flow 2: download every pending transcript of the episode, archive it
    /// next to the episode's audio file, and parse it when the format is
    /// recognized. Always returns `Ok(())` — per-transcript failures are
    /// recorded on the transcript row itself, never bubbled up, so one broken
    /// feed URL can never abort processing of the episode's other
    /// transcripts.
    pub fn process_pending_for_episode(&self, episode: &PodcastEpisode) -> Result<(), CustomError> {
        self.process_pending_for_episode_inner(episode)
    }

    /// Like [`Self::process_pending_for_episode`], but for the moment right
    /// after the episode's audio file was written locally, before the DB row
    /// carries the path: uses `local_audio_path` to derive the archive
    /// location instead of relying on `episode.file_episode_path` (which, for
    /// a first-time download, is only persisted to the DB afterwards and is
    /// never mutated on the in-memory entity the caller holds).
    pub fn process_pending_after_download(
        &self,
        episode: &PodcastEpisode,
        local_audio_path: &str,
    ) -> Result<(), CustomError> {
        let mut episode_with_path = episode.clone();
        episode_with_path.file_episode_path = Some(local_audio_path.to_string());
        self.process_pending_for_episode_inner(&episode_with_path)
    }

    fn process_pending_for_episode_inner(&self, episode: &PodcastEpisode) -> Result<(), CustomError> {
        let episode_id = parse_episode_id(&episode.id)?;
        let pending: Vec<PodcastEpisodeTranscript> = self
            .transcript_repo
            .get_by_episode_id(episode_id)?
            .into_iter()
            .filter(|t| t.status == TranscriptStatus::Pending)
            .collect();

        for transcript in &pending {
            if let Err(err) = self.download_and_archive_one(transcript, episode) {
                tracing::error!(
                    "Transcript {} for episode {} failed: {}",
                    transcript.id,
                    episode_id,
                    err
                );
                let _ = self
                    .transcript_repo
                    .set_status(transcript.id, TranscriptStatus::Failed, Some(&err.to_string()));
            }
        }

        self.recompute_preferred(episode_id)
    }

    /// Downloads, archives, and (when the format is recognized) parses one
    /// pending transcript. Any error here is caught by the caller and turned
    /// into `status = 'failed'` — this function is allowed to return `Err`
    /// freely.
    fn download_and_archive_one(
        &self,
        transcript: &PodcastEpisodeTranscript,
        episode: &PodcastEpisode,
    ) -> Result<(), CustomError> {
        let url = transcript.original_url.as_deref().ok_or_else(|| {
            CustomError::from(CustomErrorInner::Conflict(
                "transcript has no source url to download".to_string(),
                ErrorSeverity::Warning,
            ))
        })?;

        let bytes = download_bytes(url)?;

        let format = TranscriptFormat::detect(&transcript.mime_type, Some(url));
        let extension = archive_extension(format, &transcript.mime_type, Some(url));
        let archive_path = archive_path_for(episode, &extension)?;

        let mut bytes_for_write = bytes.clone();
        FileHandleWrapper::write_file(
            &archive_path,
            &mut bytes_for_write,
            &ENVIRONMENT_SERVICE.default_file_handler,
        )?;
        self.transcript_repo.set_file_path(transcript.id, &archive_path)?;

        match format {
            None => {
                // Archived, but we don't know how to parse this format.
                self.transcript_repo
                    .set_status(transcript.id, TranscriptStatus::Downloaded, None)?;
            }
            Some(format) => match parser::parse(format, &bytes) {
                Ok(segments) => {
                    self.transcript_repo.replace_segments(transcript.id, &segments)?;
                    self.transcript_repo
                        .set_status(transcript.id, TranscriptStatus::Parsed, None)?;
                }
                Err(err) => {
                    self.transcript_repo
                        .set_status(transcript.id, TranscriptStatus::Failed, Some(&err.to_string()))?;
                }
            },
        }

        Ok(())
    }

    /// Applies the preference rules from the spec: only `status = 'parsed'`
    /// transcripts qualify; a feed transcript always beats a generated one;
    /// among feed transcripts, the better-ranked format wins (JSON > VTT >
    /// SRT > HTML), and — as the final tie-break among otherwise-equal feed
    /// transcripts — the one whose `language` matches the podcast's own
    /// `language` (compared case-insensitively on the primary subtag, e.g.
    /// `"en"` matches `"en-US"`). Recomputed from scratch every time a
    /// transcript's status changes, so it's always consistent with the
    /// current DB state.
    ///
    /// The podcast lookup is best-effort: `recompute_preferred` must never
    /// fail because of it, so a lookup error (or a podcast/transcript with no
    /// language recorded) simply drops the language tie-break for that
    /// candidate — format-rank ordering alone still decides.
    pub fn recompute_preferred(&self, episode_id: Uuid) -> Result<(), CustomError> {
        let transcripts = self.transcript_repo.get_by_episode_id(episode_id)?;

        let mut candidates: Vec<&PodcastEpisodeTranscript> = transcripts
            .iter()
            .filter(|t| t.status == TranscriptStatus::Parsed)
            .collect();

        let podcast_language =
            crate::services::podcast::service::PodcastService::get_podcast_by_episode_id(episode_id)
                .ok()
                .and_then(|podcast| podcast.language);

        candidates.sort_by_key(|t| {
            let format_rank = TranscriptFormat::detect(&t.mime_type, t.original_url.as_deref())
                .map(|f| f.preference_rank())
                .unwrap_or(u8::MAX);
            let language_mismatch = match (&podcast_language, &t.language) {
                (Some(podcast_lang), Some(transcript_lang)) => {
                    !primary_language_subtag_matches(podcast_lang, transcript_lang)
                }
                _ => false,
            };
            (t.source != TranscriptSource::Feed, format_rank, language_mismatch)
        });

        let preferred_id = candidates.first().map(|t| t.id);
        self.transcript_repo.set_preferred(episode_id, preferred_id)
    }

    /// All transcript rows for an episode (any status), used by the HTTP
    /// layer to list an episode's transcripts.
    pub fn get_by_episode_id(&self, episode_id: Uuid) -> Result<Vec<PodcastEpisodeTranscript>, CustomError> {
        self.transcript_repo.get_by_episode_id(episode_id)
    }

    /// A single transcript row by id, if it exists — used by the HTTP layer
    /// to serve an individual transcript's archived file.
    pub fn get_by_id(&self, transcript_id: Uuid) -> Result<Option<PodcastEpisodeTranscript>, CustomError> {
        self.transcript_repo.get_by_id(transcript_id)
    }

    pub fn get_preferred_segments(
        &self,
        episode_id: Uuid,
    ) -> Result<Option<(PodcastEpisodeTranscript, Vec<TranscriptSegment>)>, CustomError> {
        let preferred = self
            .transcript_repo
            .get_by_episode_id(episode_id)?
            .into_iter()
            .find(|t| t.is_preferred);

        match preferred {
            Some(transcript) => {
                let segments = self.transcript_repo.get_segments(transcript.id)?;
                Ok(Some((transcript, segments)))
            }
            None => Ok(None),
        }
    }

    /// Groups the repository's flat, rank-ordered hits by episode, keeping at
    /// most [`MAX_HITS_PER_EPISODE`] per episode and ordering the resulting
    /// groups by each group's best (highest) hit rank.
    pub fn search(
        &self,
        query: &str,
        podcast_id: Option<Uuid>,
        page: i64,
    ) -> Result<Vec<TranscriptSearchGroup>, CustomError> {
        let hits = self
            .transcript_repo
            .search(query, podcast_id, page, SEARCH_INTERNAL_PAGE_SIZE)?;

        let mut order: Vec<Uuid> = Vec::new();
        let mut grouped: HashMap<Uuid, Vec<TranscriptSearchHit>> = HashMap::new();

        for hit in hits {
            let bucket = grouped.entry(hit.episode_id).or_insert_with(|| {
                order.push(hit.episode_id);
                Vec::new()
            });
            if bucket.len() < MAX_HITS_PER_EPISODE {
                bucket.push(hit);
            }
        }

        let mut groups: Vec<TranscriptSearchGroup> = order
            .into_iter()
            .map(|episode_id| TranscriptSearchGroup {
                episode_id,
                hits: grouped.remove(&episode_id).unwrap_or_default(),
            })
            .collect();

        groups.sort_by(|a, b| {
            let rank_a = a.hits.first().map(|h| h.rank).unwrap_or(f32::MIN);
            let rank_b = b.hits.first().map(|h| h.rank).unwrap_or(f32::MIN);
            rank_b.partial_cmp(&rank_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(groups)
    }

    /// Spec rule 4: an episode needs a generated transcript job as soon as
    /// there is no transcript row that is either still in flight
    /// (`pending`) or already usable (`parsed`) — including when there is no
    /// transcript row at all. A failed feed transcript alone must not block
    /// this: it must not count as "in flight or usable".
    pub fn needs_generated_transcript(&self, episode_id: Uuid) -> Result<bool, CustomError> {
        let transcripts = self.transcript_repo.get_by_episode_id(episode_id)?;
        let has_pending_or_parsed = transcripts
            .iter()
            .any(|t| matches!(t.status, TranscriptStatus::Pending | TranscriptStatus::Parsed));
        Ok(!has_pending_or_parsed)
    }

    /// Re-parses every already-archived transcript (any row with a
    /// `file_path`, regardless of current status) from its archived bytes on
    /// disk, without re-downloading. Useful after a parser bug fix: a
    /// transcript previously archived-but-unparsed (or archived-but-failed)
    /// can become `parsed` without hitting the network again. Recomputes
    /// preference for every affected episode once reparsing is done.
    ///
    /// Reads the archive straight off local disk (`std::fs::read`), matching
    /// how the rest of the codebase already treats archived/downloaded files
    /// as plain local paths (id3 tag reading, chapter parsing, NFO
    /// generation) — this does not support the S3 storage backend, the same
    /// pre-existing limitation those call sites have.
    pub fn reparse_all(&self) -> Result<ReparseReport, CustomError> {
        let transcripts = self.transcript_repo.get_all()?;
        let mut report = ReparseReport::default();
        let mut affected_episodes: HashSet<Uuid> = HashSet::new();

        for transcript in transcripts.iter().filter(|t| t.file_path.is_some()) {
            affected_episodes.insert(transcript.episode_id);
            match self.reparse_one(transcript) {
                Ok(()) => report.reparsed += 1,
                Err(err) => {
                    report.failed += 1;
                    tracing::error!("Reparse failed for transcript {}: {}", transcript.id, err);
                    let _ = self
                        .transcript_repo
                        .set_status(transcript.id, TranscriptStatus::Failed, Some(&err.to_string()));
                }
            }
        }

        for episode_id in affected_episodes {
            self.recompute_preferred(episode_id)?;
        }

        Ok(report)
    }

    fn reparse_one(&self, transcript: &PodcastEpisodeTranscript) -> Result<(), CustomError> {
        let file_path = transcript
            .file_path
            .as_deref()
            .expect("caller filters for Some file_path");

        let bytes = std::fs::read(file_path).map_err(|err| {
            CustomError::from(CustomErrorInner::Conflict(
                format!("could not read archived transcript {file_path}: {err}"),
                ErrorSeverity::Warning,
            ))
        })?;

        let format = TranscriptFormat::detect(&transcript.mime_type, transcript.original_url.as_deref())
            .ok_or_else(|| {
                CustomError::from(CustomErrorInner::Conflict(
                    "cannot determine transcript format for reparse".to_string(),
                    ErrorSeverity::Warning,
                ))
            })?;

        let segments = parser::parse(format, &bytes)
            .map_err(|err| CustomError::from(CustomErrorInner::Conflict(err.to_string(), ErrorSeverity::Warning)))?;

        self.transcript_repo.replace_segments(transcript.id, &segments)?;
        self.transcript_repo
            .set_status(transcript.id, TranscriptStatus::Parsed, None)?;
        Ok(())
    }

    /// Thin wrapper over [`TranscriptionJobRepository::enqueue`]; `None` means
    /// a job already exists for the episode.
    pub fn enqueue_job(&self, episode_id: Uuid) -> Result<Option<TranscriptionJob>, CustomError> {
        self.job_repo.enqueue(episode_id)
    }

    /// Persists a freshly Whisper-generated transcript for `episode`: writes
    /// the segments out as a VTT file archived next to the episode's audio
    /// (mirroring how feed transcripts are archived), upserts the episode's
    /// single `source = 'generated'` transcript row, replaces its segments,
    /// marks it `parsed`, and recomputes the episode's preferred transcript.
    ///
    /// Called by the job worker right after a successful
    /// [`crate::services::transcript::whisper_client::WhisperClient::transcribe`]
    /// call — any error here (e.g. no local audio path yet) is surfaced to the
    /// caller so the job can be retried/failed, unlike the feed-download path
    /// which swallows per-transcript errors itself.
    pub fn store_generated(
        &self,
        episode: &PodcastEpisode,
        segments: Vec<TranscriptSegment>,
        language: Option<String>,
    ) -> Result<(), CustomError> {
        let episode_id = parse_episode_id(&episode.id)?;
        let archive_path = archive_path_for(episode, "vtt")?;

        let vtt = whisper_client::segments_to_vtt(&segments);
        let mut bytes = vtt.into_bytes();
        FileHandleWrapper::write_file(&archive_path, &mut bytes, &ENVIRONMENT_SERVICE.default_file_handler)?;

        let transcript_id = self.transcript_repo.upsert(UpsertTranscript {
            episode_id,
            source: TranscriptSource::Generated,
            original_url: None,
            mime_type: "text/vtt".to_string(),
            language,
        })?;
        self.transcript_repo.set_file_path(transcript_id, &archive_path)?;
        self.transcript_repo.replace_segments(transcript_id, &segments)?;
        self.transcript_repo
            .set_status(transcript_id, TranscriptStatus::Parsed, None)?;

        self.recompute_preferred(episode_id)
    }
}

/// Compares two BCP-47-ish language tags on their primary subtag only, case
/// insensitively — e.g. `"en"` matches `"en-US"`, `"de-DE"` matches `"de"`.
fn primary_language_subtag_matches(a: &str, b: &str) -> bool {
    fn primary_subtag(s: &str) -> String {
        s.split(['-', '_']).next().unwrap_or(s).to_ascii_lowercase()
    }
    primary_subtag(a) == primary_subtag(b)
}

fn parse_episode_id(id: &str) -> Result<Uuid, CustomError> {
    Uuid::parse_str(id).map_err(|_| {
        CustomError::from(CustomErrorInner::BadRequest(
            "invalid episode id".to_string(),
            ErrorSeverity::Warning,
        ))
    })
}

/// Blocking GET with a hard timeout and a hard size cap. Reads at most
/// `MAX_TRANSCRIPT_BYTES + 1` bytes so an oversized body is detected without
/// buffering an unbounded response into memory.
fn download_bytes(url: &str) -> Result<Vec<u8>, CustomError> {
    let client = reqwest::blocking::Client::builder()
        .timeout(DOWNLOAD_TIMEOUT)
        .build()
        .map_err(map_reqwest_error)?;

    let response = client.get(url).send().map_err(map_reqwest_error)?;
    let status = response.status();
    if !status.is_success() {
        return Err(CustomErrorInner::Conflict(
            format!("transcript download failed with HTTP status {status}"),
            ErrorSeverity::Warning,
        )
        .into());
    }

    let mut limited = response.take((MAX_TRANSCRIPT_BYTES + 1) as u64);
    let mut buf = Vec::new();
    limited.read_to_end(&mut buf).map_err(|err| {
        CustomError::from(CustomErrorInner::Conflict(
            format!("failed reading transcript body: {err}"),
            ErrorSeverity::Warning,
        ))
    })?;

    if buf.len() > MAX_TRANSCRIPT_BYTES {
        return Err(CustomErrorInner::Conflict(
            format!("transcript exceeds the {MAX_TRANSCRIPT_BYTES}-byte size limit"),
            ErrorSeverity::Warning,
        )
        .into());
    }

    Ok(buf)
}

/// Extension used for the archived copy of a transcript: derived from the
/// recognized format when there is one; otherwise best-effort guessed from
/// the URL's own extension or the mime type's subtype, falling back to
/// `"bin"` when nothing usable is available.
fn archive_extension(format: Option<TranscriptFormat>, mime_type: &str, url: Option<&str>) -> String {
    if let Some(format) = format {
        return format_extension(format).to_string();
    }
    if let Some(ext) = url.and_then(url_extension) {
        return ext;
    }
    mime_subtype(mime_type).unwrap_or_else(|| "bin".to_string())
}

fn format_extension(format: TranscriptFormat) -> &'static str {
    match format {
        TranscriptFormat::Json => "json",
        TranscriptFormat::Vtt => "vtt",
        TranscriptFormat::Srt => "srt",
        TranscriptFormat::Html => "html",
    }
}

fn url_extension(url: &str) -> Option<String> {
    let path = url.split(['?', '#']).next().unwrap_or(url);
    let (_, ext) = path.rsplit_once('.')?;
    if ext.is_empty() || ext.contains('/') {
        None
    } else {
        Some(ext.to_ascii_lowercase())
    }
}

fn mime_subtype(mime_type: &str) -> Option<String> {
    let primary = mime_type.split(';').next().unwrap_or(mime_type).trim();
    let (_, subtype) = primary.split_once('/')?;
    if subtype.is_empty() {
        None
    } else {
        Some(subtype.to_ascii_lowercase())
    }
}

/// Builds the archive path `<episode file stem>.transcript.<ext>` next to the
/// episode's audio file. Requires `episode.file_episode_path` to already be
/// set — when the episode has no local audio file yet, there is nowhere
/// well-defined to put the transcript archive (this service is only
/// constructed with the transcript/job repositories, not a podcast/settings
/// repository, so it cannot independently replicate the download service's
/// pre-download directory-naming logic), so this fails with a clear error
/// rather than guessing a path.
fn archive_path_for(episode: &PodcastEpisode, extension: &str) -> Result<String, CustomError> {
    let file_path = episode.file_episode_path.as_deref().ok_or_else(|| {
        CustomError::from(CustomErrorInner::Conflict(
            "cannot archive transcript: episode has no downloaded audio file yet".to_string(),
            ErrorSeverity::Warning,
        ))
    })?;

    let stem = strip_extension(file_path);
    Ok(format!("{stem}.transcript.{extension}"))
}

fn strip_extension(path: &str) -> &str {
    match path.rfind('.') {
        Some(dot_idx) if dot_idx > path.rfind('/').unwrap_or(0) => &path[..dot_idx],
        _ => path,
    }
}

#[cfg(all(test, feature = "sqlite"))]
mod tests {
    use super::*;
    use crate::test_support::tests::{GLOBAL_MUTEX, ensure_test_env_vars};
    use axum::Router;
    use axum::http::StatusCode;
    use axum::routing::get;
    use diesel::prelude::*;
    use podfetch_domain::podcast_episode_transcript::TranscriptSearchHit;
    use podfetch_persistence::db::{get_connection, run_migrations};
    use podfetch_persistence::schema::{podcast_episodes, podcasts};
    use std::sync::MutexGuard;

    // ── shared DB test setup ────────────────────────────────────────────

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
        language: Option<String>,
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
        seed_podcast_with_language(None)
    }

    fn seed_podcast_with_language(language: Option<&str>) -> Uuid {
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
                language: language.map(|s| s.to_string()),
            })
            .execute(&mut get_connection())
            .expect("seed podcast");
        id
    }

    fn seed_episode(podcast_id: Uuid, file_episode_path: Option<&str>) -> PodcastEpisode {
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

        let mut episode = PodcastEpisode::default();
        episode.id = id.to_string();
        episode.podcast_id = podcast_id.to_string();
        episode.name = "Test Episode".to_string();
        episode.file_episode_path = file_episode_path.map(|s| s.to_string());
        episode
    }

    fn service() -> TranscriptService {
        TranscriptService::default_service()
    }

    // ── recompute_preferred (Step 1/2) ───────────────────────────────────

    #[test]
    fn recompute_preferred_generated_only_becomes_preferred() {
        let _guard = lock_and_prepare_db();
        let svc = service();
        let podcast_id = seed_podcast();
        let episode = seed_episode(podcast_id, None);
        let episode_id = Uuid::parse_str(&episode.id).unwrap();

        let repo = PodcastEpisodeTranscriptRepositoryImpl::new(database());
        let generated_id = repo
            .upsert(UpsertTranscript {
                episode_id,
                source: TranscriptSource::Generated,
                original_url: None,
                mime_type: "application/json".to_string(),
                language: Some("en".to_string()),
            })
            .unwrap();
        repo.set_status(generated_id, TranscriptStatus::Parsed, None).unwrap();

        svc.recompute_preferred(episode_id).unwrap();

        let (preferred, _) = svc.get_preferred_segments(episode_id).unwrap().expect("a preferred transcript");
        assert_eq!(preferred.id, generated_id);
    }

    #[test]
    fn recompute_preferred_feed_beats_generated_once_parsed() {
        let _guard = lock_and_prepare_db();
        let svc = service();
        let podcast_id = seed_podcast();
        let episode = seed_episode(podcast_id, None);
        let episode_id = Uuid::parse_str(&episode.id).unwrap();

        let repo = PodcastEpisodeTranscriptRepositoryImpl::new(database());
        let generated_id = repo
            .upsert(UpsertTranscript {
                episode_id,
                source: TranscriptSource::Generated,
                original_url: None,
                mime_type: "application/json".to_string(),
                language: Some("en".to_string()),
            })
            .unwrap();
        repo.set_status(generated_id, TranscriptStatus::Parsed, None).unwrap();
        svc.recompute_preferred(episode_id).unwrap();
        assert_eq!(svc.get_preferred_segments(episode_id).unwrap().unwrap().0.id, generated_id);

        // Now a feed transcript arrives and gets parsed too.
        let feed_id = repo
            .upsert(UpsertTranscript {
                episode_id,
                source: TranscriptSource::Feed,
                original_url: Some("https://example.com/feed.srt".to_string()),
                mime_type: "application/srt".to_string(),
                language: Some("en".to_string()),
            })
            .unwrap();
        repo.set_status(feed_id, TranscriptStatus::Parsed, None).unwrap();

        svc.recompute_preferred(episode_id).unwrap();

        let preferred = svc.get_preferred_segments(episode_id).unwrap().unwrap().0;
        assert_eq!(preferred.id, feed_id, "feed transcript must win over generated once parsed");
    }

    #[test]
    fn recompute_preferred_failed_feed_does_not_block_generated_fallback() {
        let _guard = lock_and_prepare_db();
        let svc = service();
        let podcast_id = seed_podcast();
        let episode = seed_episode(podcast_id, None);
        let episode_id = Uuid::parse_str(&episode.id).unwrap();

        let repo = PodcastEpisodeTranscriptRepositoryImpl::new(database());
        let feed_id = repo
            .upsert(UpsertTranscript {
                episode_id,
                source: TranscriptSource::Feed,
                original_url: Some("https://example.com/feed.vtt".to_string()),
                mime_type: "text/vtt".to_string(),
                language: Some("en".to_string()),
            })
            .unwrap();
        repo.set_status(feed_id, TranscriptStatus::Failed, Some("boom")).unwrap();

        let generated_id = repo
            .upsert(UpsertTranscript {
                episode_id,
                source: TranscriptSource::Generated,
                original_url: None,
                mime_type: "application/json".to_string(),
                language: Some("en".to_string()),
            })
            .unwrap();
        repo.set_status(generated_id, TranscriptStatus::Parsed, None).unwrap();

        svc.recompute_preferred(episode_id).unwrap();

        let preferred = svc.get_preferred_segments(episode_id).unwrap().unwrap().0;
        assert_eq!(preferred.id, generated_id, "failed feed transcript must not block the generated fallback");
    }

    #[test]
    fn recompute_preferred_json_beats_vtt_within_feed() {
        let _guard = lock_and_prepare_db();
        let svc = service();
        let podcast_id = seed_podcast();
        let episode = seed_episode(podcast_id, None);
        let episode_id = Uuid::parse_str(&episode.id).unwrap();

        let repo = PodcastEpisodeTranscriptRepositoryImpl::new(database());
        let vtt_id = repo
            .upsert(UpsertTranscript {
                episode_id,
                source: TranscriptSource::Feed,
                original_url: Some("https://example.com/feed.vtt".to_string()),
                mime_type: "text/vtt".to_string(),
                language: Some("en".to_string()),
            })
            .unwrap();
        repo.set_status(vtt_id, TranscriptStatus::Parsed, None).unwrap();

        let json_id = repo
            .upsert(UpsertTranscript {
                episode_id,
                source: TranscriptSource::Feed,
                original_url: Some("https://example.com/feed.json".to_string()),
                mime_type: "application/json".to_string(),
                language: Some("en".to_string()),
            })
            .unwrap();
        repo.set_status(json_id, TranscriptStatus::Parsed, None).unwrap();

        svc.recompute_preferred(episode_id).unwrap();

        let preferred = svc.get_preferred_segments(episode_id).unwrap().unwrap().0;
        assert_eq!(preferred.id, json_id, "JSON must win over VTT among parsed feed transcripts");
    }

    #[test]
    fn recompute_preferred_breaks_format_tie_by_podcast_language_match() {
        let _guard = lock_and_prepare_db();
        let svc = service();
        let podcast_id = seed_podcast_with_language(Some("de"));
        let episode = seed_episode(podcast_id, None);
        let episode_id = Uuid::parse_str(&episode.id).unwrap();

        let repo = PodcastEpisodeTranscriptRepositoryImpl::new(database());
        let de_id = repo
            .upsert(UpsertTranscript {
                episode_id,
                source: TranscriptSource::Feed,
                original_url: Some("https://example.com/feed.de.vtt".to_string()),
                mime_type: "text/vtt".to_string(),
                language: Some("de".to_string()),
            })
            .unwrap();
        repo.set_status(de_id, TranscriptStatus::Parsed, None).unwrap();

        let en_id = repo
            .upsert(UpsertTranscript {
                episode_id,
                source: TranscriptSource::Feed,
                original_url: Some("https://example.com/feed.en.vtt".to_string()),
                mime_type: "text/vtt".to_string(),
                language: Some("en".to_string()),
            })
            .unwrap();
        repo.set_status(en_id, TranscriptStatus::Parsed, None).unwrap();

        svc.recompute_preferred(episode_id).unwrap();

        let preferred = svc.get_preferred_segments(episode_id).unwrap().unwrap().0;
        assert_eq!(
            preferred.id, de_id,
            "same format+source: the transcript matching the podcast's language must win"
        );
    }

    // ── process_pending_for_episode (Step 3/4) ───────────────────────────

    fn spawn_mock_server(app: Router) -> String {
        let (addr_tx, addr_rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("build mock server runtime");
            rt.block_on(async move {
                let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                    .await
                    .expect("bind mock transcript server");
                addr_tx.send(listener.local_addr().unwrap()).unwrap();
                axum::serve(listener, app).await.unwrap();
            });
        });
        let addr = addr_rx.recv().expect("mock server address");
        format!("http://{addr}")
    }

    fn temp_episode_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("podfetch-transcript-test-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("create temp episode dir");
        dir
    }

    const SAMPLE_VTT: &str = "WEBVTT\n\n00:00:00.500 --> 00:00:04.200\nHello world\n";

    #[test]
    fn process_pending_downloads_archives_and_parses_a_recognized_format() {
        let _guard = lock_and_prepare_db();
        let svc = service();
        let podcast_id = seed_podcast();

        let dir = temp_episode_dir();
        let episode_audio_path = dir.join("episode.mp3");
        std::fs::write(&episode_audio_path, b"fake-audio").unwrap();
        let episode = seed_episode(podcast_id, Some(episode_audio_path.to_str().unwrap()));
        let episode_id = Uuid::parse_str(&episode.id).unwrap();

        let app = Router::new().route(
            "/sample.vtt",
            get(|| async { ([("content-type", "text/vtt")], SAMPLE_VTT) }),
        );
        let base_url = spawn_mock_server(app);

        let repo = PodcastEpisodeTranscriptRepositoryImpl::new(database());
        let transcript_id = repo
            .upsert(UpsertTranscript {
                episode_id,
                source: TranscriptSource::Feed,
                original_url: Some(format!("{base_url}/sample.vtt")),
                mime_type: "text/vtt".to_string(),
                language: Some("en".to_string()),
            })
            .unwrap();

        svc.process_pending_for_episode(&episode).expect("process_pending_for_episode");

        let updated = repo.get_by_id(transcript_id).unwrap().expect("row exists");
        assert_eq!(updated.status, TranscriptStatus::Parsed);
        assert!(updated.is_preferred);
        let expected_path = dir.join("episode.transcript.vtt");
        assert_eq!(updated.file_path.as_deref(), expected_path.to_str());
        assert!(expected_path.exists(), "archived transcript file must exist on disk");

        let segments = repo.get_segments(transcript_id).unwrap();
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text, "Hello world");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn process_pending_marks_failed_on_404_but_still_returns_ok() {
        let _guard = lock_and_prepare_db();
        let svc = service();
        let podcast_id = seed_podcast();

        let dir = temp_episode_dir();
        let episode_audio_path = dir.join("episode.mp3");
        std::fs::write(&episode_audio_path, b"fake-audio").unwrap();
        let episode = seed_episode(podcast_id, Some(episode_audio_path.to_str().unwrap()));
        let episode_id = Uuid::parse_str(&episode.id).unwrap();

        let app = Router::new().route("/missing.vtt", get(|| async { StatusCode::NOT_FOUND }));
        let base_url = spawn_mock_server(app);

        let repo = PodcastEpisodeTranscriptRepositoryImpl::new(database());
        let transcript_id = repo
            .upsert(UpsertTranscript {
                episode_id,
                source: TranscriptSource::Feed,
                original_url: Some(format!("{base_url}/missing.vtt")),
                mime_type: "text/vtt".to_string(),
                language: Some("en".to_string()),
            })
            .unwrap();

        let result = svc.process_pending_for_episode(&episode);
        assert!(result.is_ok(), "process_pending_for_episode must return Ok(()) even on a 404");

        let updated = repo.get_by_id(transcript_id).unwrap().expect("row exists");
        assert_eq!(updated.status, TranscriptStatus::Failed);
        assert!(updated.error.is_some());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn process_pending_marks_failed_when_download_exceeds_size_limit() {
        let _guard = lock_and_prepare_db();
        let svc = service();
        let podcast_id = seed_podcast();

        let dir = temp_episode_dir();
        let episode_audio_path = dir.join("episode.mp3");
        std::fs::write(&episode_audio_path, b"fake-audio").unwrap();
        let episode = seed_episode(podcast_id, Some(episode_audio_path.to_str().unwrap()));
        let episode_id = Uuid::parse_str(&episode.id).unwrap();

        let app = Router::new().route(
            "/huge.vtt",
            get(|| async {
                let body = vec![b'a'; MAX_TRANSCRIPT_BYTES + 1024];
                ([("content-type", "text/vtt")], body)
            }),
        );
        let base_url = spawn_mock_server(app);

        let repo = PodcastEpisodeTranscriptRepositoryImpl::new(database());
        let transcript_id = repo
            .upsert(UpsertTranscript {
                episode_id,
                source: TranscriptSource::Feed,
                original_url: Some(format!("{base_url}/huge.vtt")),
                mime_type: "text/vtt".to_string(),
                language: Some("en".to_string()),
            })
            .unwrap();

        let result = svc.process_pending_for_episode(&episode);
        assert!(result.is_ok());

        let updated = repo.get_by_id(transcript_id).unwrap().expect("row exists");
        assert_eq!(updated.status, TranscriptStatus::Failed);
        assert!(updated.error.is_some());
        assert!(updated.file_path.is_none(), "an oversized download must not be archived");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn process_pending_archives_only_for_unrecognized_format() {
        let _guard = lock_and_prepare_db();
        let svc = service();
        let podcast_id = seed_podcast();

        let dir = temp_episode_dir();
        let episode_audio_path = dir.join("episode.mp3");
        std::fs::write(&episode_audio_path, b"fake-audio").unwrap();
        let episode = seed_episode(podcast_id, Some(episode_audio_path.to_str().unwrap()));
        let episode_id = Uuid::parse_str(&episode.id).unwrap();

        let app = Router::new().route(
            "/unknown.pdf",
            get(|| async { ([("content-type", "application/pdf")], "not a real transcript") }),
        );
        let base_url = spawn_mock_server(app);

        let repo = PodcastEpisodeTranscriptRepositoryImpl::new(database());
        // The transcript's own recorded mime type (as read from the feed tag,
        // via upsert_from_feed in Flow 1) drives format detection here — not
        // the mock server's response header — since that's what the real
        // Flow 1 -> Flow 2 pipeline persists.
        let transcript_id = repo
            .upsert(UpsertTranscript {
                episode_id,
                source: TranscriptSource::Feed,
                original_url: Some(format!("{base_url}/unknown.pdf")),
                mime_type: "application/pdf".to_string(),
                language: Some("en".to_string()),
            })
            .unwrap();

        svc.process_pending_for_episode(&episode).unwrap();

        let updated = repo.get_by_id(transcript_id).unwrap().expect("row exists");
        assert_eq!(updated.status, TranscriptStatus::Downloaded);
        assert!(updated.file_path.is_some(), "unknown format must still be archived");
        assert!(!updated.is_preferred, "an archived-only transcript is never preferred");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn process_pending_for_episode_fails_when_episode_has_no_local_audio_path_yet() {
        // Documents the pre-fix behavior that `process_pending_after_download`
        // exists to work around: on a first-time download the in-memory
        // episode entity has no `file_episode_path` yet (it's only persisted
        // to the DB afterwards), so calling the plain
        // `process_pending_for_episode` at that point can never archive
        // anything and every pending transcript is permanently marked failed.
        let _guard = lock_and_prepare_db();
        let svc = service();
        let podcast_id = seed_podcast();
        let episode = seed_episode(podcast_id, None);
        let episode_id = Uuid::parse_str(&episode.id).unwrap();

        let app = Router::new().route(
            "/sample.vtt",
            get(|| async { ([("content-type", "text/vtt")], SAMPLE_VTT) }),
        );
        let base_url = spawn_mock_server(app);

        let repo = PodcastEpisodeTranscriptRepositoryImpl::new(database());
        let transcript_id = repo
            .upsert(UpsertTranscript {
                episode_id,
                source: TranscriptSource::Feed,
                original_url: Some(format!("{base_url}/sample.vtt")),
                mime_type: "text/vtt".to_string(),
                language: Some("en".to_string()),
            })
            .unwrap();

        svc.process_pending_for_episode(&episode).expect("must still return Ok(()) per-transcript");

        let updated = repo.get_by_id(transcript_id).unwrap().expect("row exists");
        assert_eq!(
            updated.status,
            TranscriptStatus::Failed,
            "without a local audio path, the transcript can never be archived and is left failed"
        );
    }

    #[test]
    fn process_pending_after_download_uses_the_freshly_written_audio_path() {
        let _guard = lock_and_prepare_db();
        let svc = service();
        let podcast_id = seed_podcast();

        // Simulates the exact moment the download hook calls this: the audio
        // file has just been written to disk, but the DB row (and the
        // in-memory entity the caller holds) still has no file_episode_path.
        let dir = temp_episode_dir();
        let episode_audio_path = dir.join("episode.mp3");
        std::fs::write(&episode_audio_path, b"fake-audio").unwrap();
        let episode = seed_episode(podcast_id, None);
        let episode_id = Uuid::parse_str(&episode.id).unwrap();
        assert!(
            episode.file_episode_path.is_none(),
            "precondition: in-memory entity must not carry the path yet"
        );

        let app = Router::new().route(
            "/sample.vtt",
            get(|| async { ([("content-type", "text/vtt")], SAMPLE_VTT) }),
        );
        let base_url = spawn_mock_server(app);

        let repo = PodcastEpisodeTranscriptRepositoryImpl::new(database());
        let transcript_id = repo
            .upsert(UpsertTranscript {
                episode_id,
                source: TranscriptSource::Feed,
                original_url: Some(format!("{base_url}/sample.vtt")),
                mime_type: "text/vtt".to_string(),
                language: Some("en".to_string()),
            })
            .unwrap();

        svc.process_pending_after_download(&episode, episode_audio_path.to_str().unwrap())
            .expect("process_pending_after_download");

        let updated = repo.get_by_id(transcript_id).unwrap().expect("row exists");
        assert_eq!(updated.status, TranscriptStatus::Parsed);
        assert!(updated.is_preferred);
        let expected_path = dir.join("episode.transcript.vtt");
        assert_eq!(updated.file_path.as_deref(), expected_path.to_str());
        assert!(expected_path.exists(), "archived transcript file must exist on disk");

        let segments = repo.get_segments(transcript_id).unwrap();
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text, "Hello world");

        let _ = std::fs::remove_dir_all(&dir);
    }

    // ── search grouping (Step 5) ──────────────────────────────────────────

    struct StubTranscriptRepo {
        hits: Vec<TranscriptSearchHit>,
    }

    impl PodcastEpisodeTranscriptRepository for StubTranscriptRepo {
        type Error = CustomError;

        fn upsert(&self, _transcript: UpsertTranscript) -> Result<Uuid, Self::Error> {
            unimplemented!("not needed for the search grouping test")
        }
        fn get_by_episode_id(&self, _episode_id: Uuid) -> Result<Vec<PodcastEpisodeTranscript>, Self::Error> {
            unimplemented!("not needed for the search grouping test")
        }
        fn get_all(&self) -> Result<Vec<PodcastEpisodeTranscript>, Self::Error> {
            unimplemented!("not needed for the search grouping test")
        }
        fn get_by_id(&self, _id: Uuid) -> Result<Option<PodcastEpisodeTranscript>, Self::Error> {
            unimplemented!("not needed for the search grouping test")
        }
        fn set_file_path(&self, _id: Uuid, _file_path: &str) -> Result<(), Self::Error> {
            unimplemented!("not needed for the search grouping test")
        }
        fn set_status(&self, _id: Uuid, _status: TranscriptStatus, _error: Option<&str>) -> Result<(), Self::Error> {
            unimplemented!("not needed for the search grouping test")
        }
        fn set_preferred(&self, _episode_id: Uuid, _preferred_id: Option<Uuid>) -> Result<(), Self::Error> {
            unimplemented!("not needed for the search grouping test")
        }
        fn replace_segments(&self, _transcript_id: Uuid, _segments: &[TranscriptSegment]) -> Result<(), Self::Error> {
            unimplemented!("not needed for the search grouping test")
        }
        fn get_segments(&self, _transcript_id: Uuid) -> Result<Vec<TranscriptSegment>, Self::Error> {
            unimplemented!("not needed for the search grouping test")
        }
        fn search(
            &self,
            _query: &str,
            _podcast_id: Option<Uuid>,
            _page: i64,
            _page_size: i64,
        ) -> Result<Vec<TranscriptSearchHit>, Self::Error> {
            Ok(self.hits.clone())
        }
    }

    struct StubJobRepo;
    impl TranscriptionJobRepository for StubJobRepo {
        type Error = CustomError;
        fn enqueue(&self, _episode_id: Uuid) -> Result<Option<TranscriptionJob>, Self::Error> {
            unimplemented!()
        }
        fn next_pending(&self) -> Result<Option<TranscriptionJob>, Self::Error> {
            unimplemented!()
        }
        fn set_status(
            &self,
            _id: Uuid,
            _status: podfetch_domain::podcast_episode_transcript::TranscriptionJobStatus,
            _error: Option<&str>,
        ) -> Result<(), Self::Error> {
            unimplemented!()
        }
        fn increment_attempts(&self, _id: Uuid) -> Result<i32, Self::Error> {
            unimplemented!()
        }
        fn reset_running_to_pending(&self) -> Result<usize, Self::Error> {
            unimplemented!()
        }
        fn get_by_episode_id(&self, _episode_id: Uuid) -> Result<Option<TranscriptionJob>, Self::Error> {
            unimplemented!()
        }
    }

    fn hit(episode_id: Uuid, rank: f32, snippet: &str) -> TranscriptSearchHit {
        TranscriptSearchHit {
            episode_id,
            transcript_id: Uuid::new_v4(),
            start_ms: Some(0),
            snippet: snippet.to_string(),
            rank,
        }
    }

    #[test]
    fn search_groups_by_episode_caps_at_three_and_orders_by_best_rank() {
        let episode_a = Uuid::new_v4();
        let episode_b = Uuid::new_v4();

        // Episode A: best rank overall (10.0) but 4 hits (one must be dropped).
        // Episode B: 3 hits, lower ranks.
        let hits = vec![
            hit(episode_a, 10.0, "a1"),
            hit(episode_b, 5.0, "b1"),
            hit(episode_a, 9.0, "a2"),
            hit(episode_b, 4.0, "b2"),
            hit(episode_a, 8.0, "a3"),
            hit(episode_b, 3.0, "b3"),
            hit(episode_a, 7.0, "a4-should-be-dropped"),
        ];
        assert_eq!(hits.len(), 7);

        let repo = Arc::new(StubTranscriptRepo { hits });
        let svc = TranscriptService::new(repo, Arc::new(StubJobRepo));

        let groups = svc.search("whatever", None, 0).unwrap();

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].episode_id, episode_a, "episode A has the best rank and must sort first");
        assert_eq!(groups[0].hits.len(), 3, "episode A must be capped at 3 hits");
        assert!(groups[0].hits.iter().all(|h| h.snippet != "a4-should-be-dropped"));
        assert_eq!(groups[1].episode_id, episode_b);
        assert_eq!(groups[1].hits.len(), 3);
    }

    // ── needs_generated_transcript + reparse_all (Step 6) ────────────────

    #[test]
    fn needs_generated_transcript_true_when_only_a_failed_feed_transcript_exists() {
        let _guard = lock_and_prepare_db();
        let svc = service();
        let podcast_id = seed_podcast();
        let episode = seed_episode(podcast_id, None);
        let episode_id = Uuid::parse_str(&episode.id).unwrap();

        let repo = PodcastEpisodeTranscriptRepositoryImpl::new(database());
        let feed_id = repo
            .upsert(UpsertTranscript {
                episode_id,
                source: TranscriptSource::Feed,
                original_url: Some("https://example.com/feed.vtt".to_string()),
                mime_type: "text/vtt".to_string(),
                language: None,
            })
            .unwrap();
        repo.set_status(feed_id, TranscriptStatus::Failed, Some("boom")).unwrap();

        assert!(svc.needs_generated_transcript(episode_id).unwrap());
    }

    #[test]
    fn needs_generated_transcript_false_when_pending_or_parsed_exists() {
        let _guard = lock_and_prepare_db();
        let svc = service();
        let podcast_id = seed_podcast();

        let episode_pending = seed_episode(podcast_id, None);
        let episode_pending_id = Uuid::parse_str(&episode_pending.id).unwrap();
        let repo = PodcastEpisodeTranscriptRepositoryImpl::new(database());
        repo.upsert(UpsertTranscript {
            episode_id: episode_pending_id,
            source: TranscriptSource::Feed,
            original_url: Some("https://example.com/feed.vtt".to_string()),
            mime_type: "text/vtt".to_string(),
            language: None,
        })
        .unwrap();
        assert!(!svc.needs_generated_transcript(episode_pending_id).unwrap());

        let episode_parsed = seed_episode(podcast_id, None);
        let episode_parsed_id = Uuid::parse_str(&episode_parsed.id).unwrap();
        let parsed_id = repo
            .upsert(UpsertTranscript {
                episode_id: episode_parsed_id,
                source: TranscriptSource::Generated,
                original_url: None,
                mime_type: "application/json".to_string(),
                language: None,
            })
            .unwrap();
        repo.set_status(parsed_id, TranscriptStatus::Parsed, None).unwrap();
        assert!(!svc.needs_generated_transcript(episode_parsed_id).unwrap());
    }

    #[test]
    fn reparse_all_reparses_archived_transcripts_and_recomputes_preference() {
        let _guard = lock_and_prepare_db();
        let svc = service();
        let podcast_id = seed_podcast();
        let episode = seed_episode(podcast_id, None);
        let episode_id = Uuid::parse_str(&episode.id).unwrap();

        let dir = temp_episode_dir();
        let archive_path = dir.join("episode.transcript.vtt");
        std::fs::write(&archive_path, SAMPLE_VTT).unwrap();

        let repo = PodcastEpisodeTranscriptRepositoryImpl::new(database());
        let transcript_id = repo
            .upsert(UpsertTranscript {
                episode_id,
                source: TranscriptSource::Feed,
                original_url: Some("https://example.com/feed.vtt".to_string()),
                mime_type: "text/vtt".to_string(),
                language: None,
            })
            .unwrap();
        repo.set_file_path(transcript_id, archive_path.to_str().unwrap()).unwrap();
        // Simulate a row that was archived but never successfully parsed.
        repo.set_status(transcript_id, TranscriptStatus::Downloaded, None).unwrap();

        let report = svc.reparse_all().unwrap();
        assert_eq!(report.reparsed, 1);
        assert_eq!(report.failed, 0);

        let updated = repo.get_by_id(transcript_id).unwrap().unwrap();
        assert_eq!(updated.status, TranscriptStatus::Parsed);
        assert!(updated.is_preferred, "reparse_all must recompute preference for affected episodes");

        let _ = std::fs::remove_dir_all(&dir);
    }

    // ── enqueue_job ───────────────────────────────────────────────────────

    #[test]
    fn enqueue_job_wraps_the_job_repository() {
        let _guard = lock_and_prepare_db();
        let svc = service();
        let podcast_id = seed_podcast();
        let episode = seed_episode(podcast_id, None);
        let episode_id = Uuid::parse_str(&episode.id).unwrap();

        let first = svc.enqueue_job(episode_id).unwrap();
        assert!(first.is_some());

        let second = svc.enqueue_job(episode_id).unwrap();
        assert!(second.is_none(), "a second enqueue for the same episode must be a no-op");
    }

    // ── upsert_from_feed (Flow 1) ─────────────────────────────────────────

    #[test]
    fn upsert_from_feed_creates_pending_feed_transcripts() {
        let _guard = lock_and_prepare_db();
        let svc = service();
        let podcast_id = seed_podcast();
        let episode = seed_episode(podcast_id, None);
        let episode_id = Uuid::parse_str(&episode.id).unwrap();

        svc.upsert_from_feed(
            episode_id,
            &[FeedTranscriptTag {
                url: "https://example.com/feed.vtt".to_string(),
                mime_type: "text/vtt".to_string(),
                language: Some("en".to_string()),
            }],
        )
        .unwrap();

        let repo = PodcastEpisodeTranscriptRepositoryImpl::new(database());
        let rows = repo.get_by_episode_id(episode_id).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].status, TranscriptStatus::Pending);
        assert_eq!(rows[0].source, TranscriptSource::Feed);
        assert_eq!(rows[0].original_url.as_deref(), Some("https://example.com/feed.vtt"));
    }

    /// This is the DB-level counterpart to Task 7's feed-parse hook
    /// (`extract_transcript_tags` + the item-loop call into
    /// `upsert_from_feed` in `usecases::podcast_episode`): a feed refresh
    /// calls `upsert_from_feed` again on every poll with the same tags read
    /// straight from the RSS item, so repeated calls for the same
    /// (episode_id, url) pair must stay idempotent — no duplicate rows, and
    /// a transcript that has already been parsed must not be bounced back to
    /// `pending` just because the feed was re-fetched.
    #[test]
    fn upsert_from_feed_is_idempotent_and_preserves_parsed_status() {
        let _guard = lock_and_prepare_db();
        let svc = service();
        let podcast_id = seed_podcast();
        let episode = seed_episode(podcast_id, None);
        let episode_id = Uuid::parse_str(&episode.id).unwrap();

        let tags = [
            FeedTranscriptTag {
                url: "https://example.com/feed.vtt".to_string(),
                mime_type: "text/vtt".to_string(),
                language: Some("en".to_string()),
            },
            FeedTranscriptTag {
                url: "https://example.com/feed.json".to_string(),
                mime_type: "application/json".to_string(),
                language: Some("en".to_string()),
            },
        ];

        svc.upsert_from_feed(episode_id, &tags).unwrap();

        let repo = PodcastEpisodeTranscriptRepositoryImpl::new(database());
        let rows = repo.get_by_episode_id(episode_id).unwrap();
        assert_eq!(rows.len(), 2);

        // Simulate Flow 2 having already downloaded and parsed one of them.
        let vtt_row = rows
            .iter()
            .find(|row| row.original_url.as_deref() == Some("https://example.com/feed.vtt"))
            .unwrap();
        repo.set_status(vtt_row.id, TranscriptStatus::Parsed, None).unwrap();

        // A second feed refresh sees the exact same tags again.
        svc.upsert_from_feed(episode_id, &tags).unwrap();

        let rows_after = repo.get_by_episode_id(episode_id).unwrap();
        assert_eq!(rows_after.len(), 2, "repeated upserts must not create duplicate rows");

        let vtt_row_after = rows_after
            .iter()
            .find(|row| row.original_url.as_deref() == Some("https://example.com/feed.vtt"))
            .unwrap();
        assert_eq!(
            vtt_row_after.status,
            TranscriptStatus::Parsed,
            "re-syncing feed tags must not reset an already-parsed transcript back to pending"
        );

        let json_row_after = rows_after
            .iter()
            .find(|row| row.original_url.as_deref() == Some("https://example.com/feed.json"))
            .unwrap();
        assert_eq!(json_row_after.status, TranscriptStatus::Pending);
    }
}
