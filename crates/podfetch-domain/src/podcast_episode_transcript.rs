use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TranscriptSource {
    Feed,
    Generated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TranscriptStatus {
    Pending,
    Downloaded,
    Parsed,
    Failed,
}

#[derive(Debug, Clone)]
pub struct PodcastEpisodeTranscript {
    pub id: Uuid,
    pub episode_id: Uuid,
    pub source: TranscriptSource,
    pub original_url: Option<String>,
    pub file_path: Option<String>,
    pub mime_type: String,
    pub language: Option<String>,
    pub is_preferred: bool,
    pub status: TranscriptStatus,
    pub error: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct UpsertTranscript {
    pub episode_id: Uuid,
    pub source: TranscriptSource,
    pub original_url: Option<String>,
    pub mime_type: String,
    pub language: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TranscriptSegment {
    pub idx: i32,
    pub start_ms: Option<i32>,
    pub end_ms: Option<i32>,
    pub speaker: Option<String>,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct TranscriptSearchHit {
    pub episode_id: Uuid,
    pub transcript_id: Uuid,
    pub start_ms: Option<i32>,
    /// Snippet with <b>…</b> highlighting produced by the database.
    pub snippet: String,
    /// Relevance score; higher is more relevant, comparable only within a single search call.
    pub rank: f32,
}

pub trait PodcastEpisodeTranscriptRepository: Send + Sync {
    type Error;
    /// Upsert keyed on (episode_id, original_url); returns the row's id.
    fn upsert(&self, transcript: UpsertTranscript) -> Result<Uuid, Self::Error>;
    fn get_by_episode_id(&self, episode_id: Uuid) -> Result<Vec<PodcastEpisodeTranscript>, Self::Error>;
    /// Every transcript row across all episodes. Used by `reparse_all`, which
    /// needs to walk the whole table rather than a single episode's rows.
    fn get_all(&self) -> Result<Vec<PodcastEpisodeTranscript>, Self::Error>;
    fn get_by_id(&self, id: Uuid) -> Result<Option<PodcastEpisodeTranscript>, Self::Error>;
    fn set_file_path(&self, id: Uuid, file_path: &str) -> Result<(), Self::Error>;
    fn set_status(&self, id: Uuid, status: TranscriptStatus, error: Option<&str>) -> Result<(), Self::Error>;
    fn set_preferred(&self, episode_id: Uuid, preferred_id: Option<Uuid>) -> Result<(), Self::Error>;
    /// Deletes the transcript's old segments and inserts the new ones in one transaction.
    fn replace_segments(&self, transcript_id: Uuid, segments: &[TranscriptSegment]) -> Result<(), Self::Error>;
    fn get_segments(&self, transcript_id: Uuid) -> Result<Vec<TranscriptSegment>, Self::Error>;
    fn search(
        &self,
        query: &str,
        podcast_id: Option<Uuid>,
        page: i64,
        page_size: i64,
    ) -> Result<Vec<TranscriptSearchHit>, Self::Error>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TranscriptionJobStatus {
    Pending,
    Running,
    Done,
    Failed,
}

#[derive(Debug, Clone)]
pub struct TranscriptionJob {
    pub id: Uuid,
    pub episode_id: Uuid,
    pub status: TranscriptionJobStatus,
    pub attempts: i32,
    pub error: Option<String>,
}

pub trait TranscriptionJobRepository: Send + Sync {
    type Error;
    /// Enqueues a job; returns Ok(None) if one already exists for the episode (UNIQUE).
    fn enqueue(&self, episode_id: Uuid) -> Result<Option<TranscriptionJob>, Self::Error>;
    fn next_pending(&self) -> Result<Option<TranscriptionJob>, Self::Error>;
    fn set_status(&self, id: Uuid, status: TranscriptionJobStatus, error: Option<&str>) -> Result<(), Self::Error>;
    fn increment_attempts(&self, id: Uuid) -> Result<i32, Self::Error>;
    fn reset_running_to_pending(&self) -> Result<usize, Self::Error>;
    fn get_by_episode_id(&self, episode_id: Uuid) -> Result<Option<TranscriptionJob>, Self::Error>;
}

// String conversion implementations

impl TranscriptSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            TranscriptSource::Feed => "feed",
            TranscriptSource::Generated => "generated",
        }
    }

    #[allow(clippy::should_implement_trait)] // fallible parse returning Option, not FromStr
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "feed" => Some(TranscriptSource::Feed),
            "generated" => Some(TranscriptSource::Generated),
            _ => None,
        }
    }
}

impl TranscriptStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TranscriptStatus::Pending => "pending",
            TranscriptStatus::Downloaded => "downloaded",
            TranscriptStatus::Parsed => "parsed",
            TranscriptStatus::Failed => "failed",
        }
    }

    #[allow(clippy::should_implement_trait)] // fallible parse returning Option, not FromStr
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(TranscriptStatus::Pending),
            "downloaded" => Some(TranscriptStatus::Downloaded),
            "parsed" => Some(TranscriptStatus::Parsed),
            "failed" => Some(TranscriptStatus::Failed),
            _ => None,
        }
    }
}

impl TranscriptionJobStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TranscriptionJobStatus::Pending => "pending",
            TranscriptionJobStatus::Running => "running",
            TranscriptionJobStatus::Done => "done",
            TranscriptionJobStatus::Failed => "failed",
        }
    }

    #[allow(clippy::should_implement_trait)] // fallible parse returning Option, not FromStr
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(TranscriptionJobStatus::Pending),
            "running" => Some(TranscriptionJobStatus::Running),
            "done" => Some(TranscriptionJobStatus::Done),
            "failed" => Some(TranscriptionJobStatus::Failed),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transcript_source_as_str() {
        assert_eq!(TranscriptSource::Feed.as_str(), "feed");
        assert_eq!(TranscriptSource::Generated.as_str(), "generated");
    }

    #[test]
    fn transcript_source_from_str() {
        assert_eq!(TranscriptSource::from_str("feed"), Some(TranscriptSource::Feed));
        assert_eq!(TranscriptSource::from_str("generated"), Some(TranscriptSource::Generated));
        assert_eq!(TranscriptSource::from_str("unknown"), None);
    }

    #[test]
    fn transcript_source_roundtrip() {
        let variants = vec![TranscriptSource::Feed, TranscriptSource::Generated];
        for variant in variants {
            assert_eq!(TranscriptSource::from_str(variant.as_str()), Some(variant));
        }
    }

    #[test]
    fn transcript_status_as_str() {
        assert_eq!(TranscriptStatus::Pending.as_str(), "pending");
        assert_eq!(TranscriptStatus::Downloaded.as_str(), "downloaded");
        assert_eq!(TranscriptStatus::Parsed.as_str(), "parsed");
        assert_eq!(TranscriptStatus::Failed.as_str(), "failed");
    }

    #[test]
    fn transcript_status_from_str() {
        assert_eq!(TranscriptStatus::from_str("pending"), Some(TranscriptStatus::Pending));
        assert_eq!(TranscriptStatus::from_str("downloaded"), Some(TranscriptStatus::Downloaded));
        assert_eq!(TranscriptStatus::from_str("parsed"), Some(TranscriptStatus::Parsed));
        assert_eq!(TranscriptStatus::from_str("failed"), Some(TranscriptStatus::Failed));
        assert_eq!(TranscriptStatus::from_str("unknown"), None);
    }

    #[test]
    fn transcript_status_roundtrip() {
        let variants = vec![
            TranscriptStatus::Pending,
            TranscriptStatus::Downloaded,
            TranscriptStatus::Parsed,
            TranscriptStatus::Failed,
        ];
        for variant in variants {
            assert_eq!(TranscriptStatus::from_str(variant.as_str()), Some(variant));
        }
    }

    #[test]
    fn transcription_job_status_as_str() {
        assert_eq!(TranscriptionJobStatus::Pending.as_str(), "pending");
        assert_eq!(TranscriptionJobStatus::Running.as_str(), "running");
        assert_eq!(TranscriptionJobStatus::Done.as_str(), "done");
        assert_eq!(TranscriptionJobStatus::Failed.as_str(), "failed");
    }

    #[test]
    fn transcription_job_status_from_str() {
        assert_eq!(TranscriptionJobStatus::from_str("pending"), Some(TranscriptionJobStatus::Pending));
        assert_eq!(TranscriptionJobStatus::from_str("running"), Some(TranscriptionJobStatus::Running));
        assert_eq!(TranscriptionJobStatus::from_str("done"), Some(TranscriptionJobStatus::Done));
        assert_eq!(TranscriptionJobStatus::from_str("failed"), Some(TranscriptionJobStatus::Failed));
        assert_eq!(TranscriptionJobStatus::from_str("unknown"), None);
    }

    #[test]
    fn transcription_job_status_roundtrip() {
        let variants = vec![
            TranscriptionJobStatus::Pending,
            TranscriptionJobStatus::Running,
            TranscriptionJobStatus::Done,
            TranscriptionJobStatus::Failed,
        ];
        for variant in variants {
            assert_eq!(TranscriptionJobStatus::from_str(variant.as_str()), Some(variant));
        }
    }
}
