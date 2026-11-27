//! Episode actions: what a user did with an episode on which device.
//!
//! This is the gPodder episode-action log and at the same time the playback
//! progress of PodFetch's own player. Actions are stored per user id; the
//! username only travels along because the gPodder protocol shows it.

use chrono::NaiveDateTime;
use uuid::Uuid;

/// The user an action belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listener {
    pub id: Uuid,
    pub username: String,
}

impl Listener {
    pub fn new(id: Uuid, username: impl Into<String>) -> Self {
        Self {
            id,
            username: username.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Episode {
    pub id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub device: String,
    pub podcast: String,
    pub episode: String,
    pub timestamp: NaiveDateTime,
    pub guid: Option<String>,
    pub action: String,
    pub started: Option<i32>,
    pub position: Option<i32>,
    pub total: Option<i32>,
}

impl Episode {
    /// Whether this action marks the episode as heard: a position is known
    /// and it reached the end, or no length was reported. Kept identical to
    /// the former SQL filter `position IS NULL OR total != position`, whose
    /// NULL semantics treat a missing total as heard.
    pub fn counts_as_heard(&self) -> bool {
        match (self.position, self.total) {
            (None, _) => false,
            (Some(_), None) => true,
            (Some(position), Some(total)) => position == total,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NewEpisode {
    pub listener: Listener,
    pub device: String,
    pub podcast: String,
    pub episode: String,
    pub timestamp: NaiveDateTime,
    pub guid: Option<String>,
    pub action: String,
    pub started: Option<i32>,
    pub position: Option<i32>,
    pub total: Option<i32>,
}

pub trait EpisodeRepository: Send + Sync {
    type Error;

    fn create(&self, episode: NewEpisode) -> Result<Episode, Self::Error>;

    /// Stores `episode` unless an identical action (same user, timestamp,
    /// device, podcast and episode URL) exists; returns the stored action.
    fn insert_episode(&self, episode: &Episode) -> Result<Episode, Self::Error>;

    fn find_duplicate_action(&self, episode: &Episode) -> Result<Option<Episode>, Self::Error>;

    /// The latest action of the listener on a device for an episode guid.
    fn find_by_device_and_guid(
        &self,
        listener: &Listener,
        device: &str,
        guid: &str,
    ) -> Result<Option<Episode>, Self::Error>;

    /// The latest action of the listener for an episode guid, any device.
    /// Used when episode URLs differ (e.g. local vs. original feed).
    fn find_latest_by_guid(
        &self,
        listener: &Listener,
        guid: &str,
    ) -> Result<Option<Episode>, Self::Error>;

    /// The latest action per guid, for every given guid that has one.
    fn find_latest_by_guids(
        &self,
        listener: &Listener,
        guids: &[String],
    ) -> Result<Vec<Episode>, Self::Error>;

    /// Every action of the listener, newest first. With `device` and
    /// `default_device`, actions of both devices are returned.
    fn find_actions(
        &self,
        listener: &Listener,
        since: Option<NaiveDateTime>,
        device: Option<&str>,
        podcast: Option<&str>,
        default_device: Option<&str>,
    ) -> Result<Vec<Episode>, Self::Error>;

    /// Per episode the latest action, if that action is a `play`; newest
    /// first.
    fn find_last_played(&self, listener: &Listener) -> Result<Vec<Episode>, Self::Error>;

    /// Every episode guid the listener has any action for.
    fn guids_with_actions(&self, listener: &Listener) -> Result<Vec<String>, Self::Error>;

    fn update_position(
        &self,
        id: Uuid,
        position: i32,
        timestamp: NaiveDateTime,
    ) -> Result<(), Self::Error>;

    /// Sets position and total (e.g. `position == total` marks it played).
    fn update_progress(
        &self,
        id: Uuid,
        position: i32,
        total: i32,
        timestamp: NaiveDateTime,
    ) -> Result<(), Self::Error>;

    fn delete_by_user(&self, user_id: Uuid) -> Result<usize, Self::Error>;

    fn delete_by_podcast_feed(&self, podcast_feed: &str) -> Result<usize, Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn action(position: Option<i32>, total: Option<i32>) -> Episode {
        Episode {
            id: Uuid::nil(),
            user_id: Uuid::nil(),
            username: "u".into(),
            device: "d".into(),
            podcast: "p".into(),
            episode: "e".into(),
            timestamp: NaiveDateTime::default(),
            guid: None,
            action: "play".into(),
            started: None,
            position,
            total,
        }
    }

    #[test]
    fn heard_matches_the_former_sql_filter() {
        assert!(!action(None, None).counts_as_heard());
        assert!(!action(None, Some(10)).counts_as_heard());
        assert!(!action(Some(5), Some(10)).counts_as_heard());
        assert!(action(Some(10), Some(10)).counts_as_heard());
        assert!(action(Some(5), None).counts_as_heard());
    }
}
