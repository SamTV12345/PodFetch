use chrono::NaiveDateTime;
use uuid::Uuid;

/// The triage decision a user has made about a podcast episode.
///
/// There is deliberately no `Inbox` variant: an episode is "in the inbox"
/// precisely when no [`EpisodeTriage`] row exists for the `(user, episode)`
/// pair. That keeps newly discovered episodes flowing into the inbox without
/// having to fan-out a row to every user on every feed refresh.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriageStatus {
    /// Picked from the inbox to listen to — shown in the waiting list.
    Queued,
    /// Listened to / kept only for the downloaded-episodes archive.
    Archived,
    /// Not interesting — removed from the inbox.
    Dismissed,
}

impl TriageStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TriageStatus::Queued => "queued",
            TriageStatus::Archived => "archived",
            TriageStatus::Dismissed => "dismissed",
        }
    }

    pub fn from_string(value: &str) -> Option<Self> {
        match value {
            "queued" => Some(TriageStatus::Queued),
            "archived" => Some(TriageStatus::Archived),
            "dismissed" => Some(TriageStatus::Dismissed),
            _ => None,
        }
    }
}

/// Per-user triage state for a single podcast episode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpisodeTriage {
    pub user_id: Uuid,
    pub episode_id: Uuid,
    pub status: TriageStatus,
    pub updated_at: NaiveDateTime,
}

/// Repository trait for per-user episode triage persistence.
pub trait EpisodeTriageRepository: Send + Sync {
    type Error;

    /// Fetch the triage row for a `(user, episode)` pair, if any.
    fn get(&self, user_id: Uuid, episode_id: Uuid) -> Result<Option<EpisodeTriage>, Self::Error>;

    /// Insert or update the triage state for a `(user, episode)` pair.
    fn upsert(&self, triage: EpisodeTriage) -> Result<(), Self::Error>;

    /// Remove the triage row, returning the episode to the user's inbox.
    fn delete(&self, user_id: Uuid, episode_id: Uuid) -> Result<usize, Self::Error>;

    /// Remove every triage row referencing an episode (e.g. on episode/podcast
    /// deletion) across all users.
    fn delete_by_episode_id(&self, episode_id: Uuid) -> Result<usize, Self::Error>;

    /// Episode ids the user has triaged into a specific status.
    fn list_episode_ids_by_status(
        &self,
        user_id: Uuid,
        status: TriageStatus,
    ) -> Result<Vec<Uuid>, Self::Error>;

    /// Every episode id the user has triaged (any status). Used to exclude
    /// already-decided episodes from the inbox.
    fn list_triaged_episode_ids(&self, user_id: Uuid) -> Result<Vec<Uuid>, Self::Error>;
}
