//! Cast targets (Chromecast and Mopidy) and who may see them.
//!
//! The rows live in the `devices` table, which is still shared with the
//! gPodder sync devices. The `kind` column tells them apart; this context
//! reads and writes castable kinds only.

use chrono::NaiveDateTime;
use uuid::Uuid;

/// Values of the `kind` column for cast targets.
pub mod kind {
    /// Personal Chromecast, visible only to its owner.
    pub const CHROMECAST_PERSONAL: &str = "chromecast_personal";
    /// Shared Chromecast, visible to every user on the instance.
    pub const CHROMECAST_SHARED: &str = "chromecast_shared";
    /// Personal Mopidy server, visible only to its owner.
    pub const MOPIDY_PERSONAL: &str = "mopidy_personal";
    /// Shared Mopidy server, visible to every user on the instance.
    pub const MOPIDY_SHARED: &str = "mopidy_shared";

    /// Every kind this context owns.
    pub const CASTABLE: [&str; 4] = [
        CHROMECAST_PERSONAL,
        CHROMECAST_SHARED,
        MOPIDY_PERSONAL,
        MOPIDY_SHARED,
    ];

    pub fn is_chromecast(kind: &str) -> bool {
        matches!(kind, CHROMECAST_PERSONAL | CHROMECAST_SHARED)
    }

    pub fn is_chromecast_shared(kind: &str) -> bool {
        kind == CHROMECAST_SHARED
    }

    pub fn is_mopidy(kind: &str) -> bool {
        matches!(kind, MOPIDY_PERSONAL | MOPIDY_SHARED)
    }

    pub fn is_castable(kind: &str) -> bool {
        CASTABLE.contains(&kind)
    }
}

/// A device that playback can be sent to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CastDevice {
    pub id: Option<Uuid>,
    pub deviceid: String,
    pub kind: String,
    pub name: String,
    pub user_id: Uuid,
    pub chromecast_uuid: Option<String>,
    pub agent_id: Option<String>,
    pub last_seen_at: Option<NaiveDateTime>,
    pub ip: Option<String>,
    /// Mopidy RPC base URL (e.g. `http://mopidy.local:6680`); `None` for Chromecasts.
    pub base_url: Option<String>,
}

pub trait CastDeviceRepository: Send + Sync {
    type Error;

    /// Stores a new cast target (e.g. a Mopidy server added in the UI).
    fn register(&self, device: CastDevice) -> Result<CastDevice, Self::Error>;

    /// Removes every cast target owned by `owner`; returns how many.
    fn delete_owned_by(&self, owner: Uuid) -> Result<usize, Self::Error>;

    /// The user's own personal targets plus every shared target.
    fn list_visible_to(&self, user_id: Uuid) -> Result<Vec<CastDevice>, Self::Error>;

    fn find_by_chromecast_uuid(
        &self,
        chromecast_uuid: &str,
    ) -> Result<Option<CastDevice>, Self::Error>;

    fn find_by_id(&self, id: Uuid) -> Result<Option<CastDevice>, Self::Error>;

    /// Deletes one target by id; returns rows removed.
    fn delete_by_id(&self, id: Uuid) -> Result<usize, Self::Error>;

    /// Inserts or updates a Chromecast reported by an agent, keyed by
    /// `chromecast_uuid`. New rows are personal devices of the agent's
    /// user; an admin can promote them to shared. The kind of an existing
    /// row is kept.
    fn upsert_chromecast_from_agent(
        &self,
        chromecast_uuid: &str,
        agent_id: &str,
        owner_user_id: Uuid,
        name: &str,
        ip: Option<&str>,
        last_seen_at: NaiveDateTime,
    ) -> Result<CastDevice, Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::kind;

    #[test]
    fn kinds_are_classified() {
        assert!(kind::is_mopidy(kind::MOPIDY_PERSONAL));
        assert!(kind::is_mopidy(kind::MOPIDY_SHARED));
        assert!(!kind::is_mopidy(kind::CHROMECAST_SHARED));
        assert!(kind::is_castable(kind::MOPIDY_SHARED));
        assert!(kind::is_castable(kind::CHROMECAST_PERSONAL));
        assert!(!kind::is_castable("desktop"));
        assert!(!kind::is_chromecast(kind::MOPIDY_PERSONAL));
    }
}
