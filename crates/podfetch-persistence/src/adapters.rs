use crate::db::Database;
use common_infrastructure::error::CustomError;
use uuid::Uuid;

// ── PodcastEpisodeChapter ───────────────────────────────────────────────────

use crate::podcast_episode_chapter::DieselPodcastEpisodeChapterRepository;
use podfetch_domain::podcast_episode_chapter::{
    PodcastEpisodeChapter, PodcastEpisodeChapterRepository, UpsertPodcastEpisodeChapter,
};

pub struct PodcastEpisodeChapterRepositoryImpl {
    inner: DieselPodcastEpisodeChapterRepository,
}

impl PodcastEpisodeChapterRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselPodcastEpisodeChapterRepository::new(database),
        }
    }
}

impl PodcastEpisodeChapterRepository for PodcastEpisodeChapterRepositoryImpl {
    type Error = CustomError;

    fn upsert(&self, chapter: UpsertPodcastEpisodeChapter) -> Result<(), Self::Error> {
        self.inner.upsert(chapter).map_err(Into::into)
    }

    fn get_by_episode_id(
        &self,
        episode_id: Uuid,
    ) -> Result<Vec<PodcastEpisodeChapter>, Self::Error> {
        self.inner.get_by_episode_id(episode_id).map_err(Into::into)
    }

    fn delete_by_episode_ids(&self, episode_ids: &[Uuid]) -> Result<(), Self::Error> {
        self.inner
            .delete_by_episode_ids(episode_ids)
            .map_err(Into::into)
    }
}

// ── PodcastSettings ─────────────────────────────────────────────────────────

use crate::podcast_settings::DieselPodcastSettingsRepository;
use podfetch_domain::podcast_settings::{PodcastSetting, PodcastSettingsRepository};

pub struct PodcastSettingsRepositoryImpl {
    inner: DieselPodcastSettingsRepository,
}

impl PodcastSettingsRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselPodcastSettingsRepository::new(database),
        }
    }
}

impl PodcastSettingsRepository for PodcastSettingsRepositoryImpl {
    type Error = CustomError;

    fn get_settings(&self, podcast_id: Uuid) -> Result<Option<PodcastSetting>, Self::Error> {
        self.inner.get_settings(podcast_id).map_err(Into::into)
    }

    fn upsert_settings(&self, setting: PodcastSetting) -> Result<PodcastSetting, Self::Error> {
        self.inner.upsert_settings(setting).map_err(Into::into)
    }

    fn delete_settings(&self, podcast_id: Uuid) -> Result<(), Self::Error> {
        self.inner.delete_settings(podcast_id).map_err(Into::into)
    }
}

// ── Settings ────────────────────────────────────────────────────────────────

use crate::settings::DieselSettingsRepository;
use podfetch_domain::settings::{Setting, SettingRepository};

pub struct SettingsRepositoryImpl {
    inner: DieselSettingsRepository,
}

impl SettingsRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselSettingsRepository::new(database),
        }
    }
}

impl SettingRepository for SettingsRepositoryImpl {
    type Error = CustomError;

    fn get_settings(&self) -> Result<Option<Setting>, Self::Error> {
        self.inner.get_settings().map_err(Into::into)
    }

    fn update_settings(&self, setting: Setting) -> Result<Setting, Self::Error> {
        self.inner.update_settings(setting).map_err(Into::into)
    }

    fn insert_default_settings(&self) -> Result<(), Self::Error> {
        self.inner.insert_default_settings().map_err(Into::into)
    }
}

// ── PodcastEpisodeTranscript ─────────────────────────────────────────────────

use crate::podcast_episode_transcript::DieselPodcastEpisodeTranscriptRepository;
use podfetch_domain::podcast_episode_transcript::{
    PodcastEpisodeTranscript, PodcastEpisodeTranscriptRepository, TranscriptSearchHit,
    TranscriptSegment, TranscriptStatus, UpsertTranscript,
};

pub struct PodcastEpisodeTranscriptRepositoryImpl {
    inner: DieselPodcastEpisodeTranscriptRepository,
}

impl PodcastEpisodeTranscriptRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselPodcastEpisodeTranscriptRepository::new(database),
        }
    }
}

impl PodcastEpisodeTranscriptRepository for PodcastEpisodeTranscriptRepositoryImpl {
    type Error = CustomError;

    fn upsert(&self, transcript: UpsertTranscript) -> Result<Uuid, Self::Error> {
        self.inner.upsert(transcript).map_err(Into::into)
    }

    fn get_by_episode_id(
        &self,
        episode_id: Uuid,
    ) -> Result<Vec<PodcastEpisodeTranscript>, Self::Error> {
        self.inner.get_by_episode_id(episode_id).map_err(Into::into)
    }

    fn get_all(&self) -> Result<Vec<PodcastEpisodeTranscript>, Self::Error> {
        self.inner.get_all().map_err(Into::into)
    }

    fn get_by_id(&self, id: Uuid) -> Result<Option<PodcastEpisodeTranscript>, Self::Error> {
        self.inner.get_by_id(id).map_err(Into::into)
    }

    fn set_file_path(&self, id: Uuid, file_path: &str) -> Result<(), Self::Error> {
        self.inner.set_file_path(id, file_path).map_err(Into::into)
    }

    fn set_status(
        &self,
        id: Uuid,
        status: TranscriptStatus,
        error: Option<&str>,
    ) -> Result<(), Self::Error> {
        self.inner.set_status(id, status, error).map_err(Into::into)
    }

    fn set_preferred(
        &self,
        episode_id: Uuid,
        preferred_id: Option<Uuid>,
    ) -> Result<(), Self::Error> {
        self.inner
            .set_preferred(episode_id, preferred_id)
            .map_err(Into::into)
    }

    fn replace_segments(
        &self,
        transcript_id: Uuid,
        segments: &[TranscriptSegment],
    ) -> Result<(), Self::Error> {
        self.inner
            .replace_segments(transcript_id, segments)
            .map_err(Into::into)
    }

    fn get_segments(&self, transcript_id: Uuid) -> Result<Vec<TranscriptSegment>, Self::Error> {
        self.inner.get_segments(transcript_id).map_err(Into::into)
    }

    fn search(
        &self,
        query: &str,
        podcast_id: Option<Uuid>,
        page: i64,
        page_size: i64,
    ) -> Result<Vec<TranscriptSearchHit>, Self::Error> {
        self.inner
            .search(query, podcast_id, page, page_size)
            .map_err(Into::into)
    }

    fn delete_by_episode_ids(&self, episode_ids: &[Uuid]) -> Result<(), Self::Error> {
        self.inner
            .delete_by_episode_ids(episode_ids)
            .map_err(Into::into)
    }
}

// ── TranscriptionJob ──────────────────────────────────────────────────────────

use crate::podcast_episode_transcript::DieselTranscriptionJobRepository;
use podfetch_domain::podcast_episode_transcript::{
    TranscriptionJob, TranscriptionJobRepository, TranscriptionJobStatus,
};

pub struct TranscriptionJobRepositoryImpl {
    inner: DieselTranscriptionJobRepository,
}

impl TranscriptionJobRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselTranscriptionJobRepository::new(database),
        }
    }
}

impl TranscriptionJobRepository for TranscriptionJobRepositoryImpl {
    type Error = CustomError;

    fn enqueue(&self, episode_id: Uuid) -> Result<Option<TranscriptionJob>, Self::Error> {
        self.inner.enqueue(episode_id).map_err(Into::into)
    }

    fn next_pending(&self) -> Result<Option<TranscriptionJob>, Self::Error> {
        self.inner.next_pending().map_err(Into::into)
    }

    fn set_status(
        &self,
        id: Uuid,
        status: TranscriptionJobStatus,
        error: Option<&str>,
    ) -> Result<(), Self::Error> {
        self.inner.set_status(id, status, error).map_err(Into::into)
    }

    fn increment_attempts(&self, id: Uuid) -> Result<i32, Self::Error> {
        self.inner.increment_attempts(id).map_err(Into::into)
    }

    fn reset_running_to_pending(&self) -> Result<usize, Self::Error> {
        self.inner.reset_running_to_pending().map_err(Into::into)
    }

    fn get_by_episode_id(&self, episode_id: Uuid) -> Result<Option<TranscriptionJob>, Self::Error> {
        self.inner.get_by_episode_id(episode_id).map_err(Into::into)
    }

    fn delete_by_episode_ids(&self, episode_ids: &[Uuid]) -> Result<(), Self::Error> {
        self.inner
            .delete_by_episode_ids(episode_ids)
            .map_err(Into::into)
    }
}
