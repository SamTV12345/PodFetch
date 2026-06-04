use chrono::NaiveDateTime;
use uuid::Uuid;

/// Discriminator values used in the `kind` column.
pub mod kind {
    /// gPodder-style desktop client (legacy). Pre-existing values kept here
    /// for symmetry; use the constants instead of stringly-typed comparisons.
    pub const DESKTOP: &str = "desktop";
    pub const LAPTOP: &str = "laptop";
    pub const MOBILE: &str = "mobile";
    pub const SERVER: &str = "server";
    pub const OTHER: &str = "other";

    /// Personal Chromecast — visible only to the owning user.
    pub const CHROMECAST_PERSONAL: &str = "chromecast_personal";
    /// Shared/household Chromecast — visible to every user on the instance.
    pub const CHROMECAST_SHARED: &str = "chromecast_shared";

    /// True for any chromecast_* kind.
    pub fn is_chromecast(kind: &str) -> bool {
        matches!(kind, CHROMECAST_PERSONAL | CHROMECAST_SHARED)
    }

    pub fn is_chromecast_shared(kind: &str) -> bool {
        kind == CHROMECAST_SHARED
    }

    /// Personal Mopidy server — visible only to the owning user.
    pub const MOPIDY_PERSONAL: &str = "mopidy_personal";
    /// Shared/household Mopidy server — visible to every user on the instance.
    pub const MOPIDY_SHARED: &str = "mopidy_shared";

    /// True for any mopidy_* kind.
    pub fn is_mopidy(kind: &str) -> bool {
        matches!(kind, MOPIDY_PERSONAL | MOPIDY_SHARED)
    }

    /// True for any kind that can be a remote-playback target (chromecast or mopidy).
    pub fn is_castable(kind: &str) -> bool {
        is_chromecast(kind) || is_mopidy(kind)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Device {
    pub id: Option<Uuid>,
    pub deviceid: String,
    pub kind: String,
    pub name: String,
    pub user_id: Uuid,
    pub chromecast_uuid: Option<String>,
    pub agent_id: Option<String>,
    pub last_seen_at: Option<NaiveDateTime>,
    pub ip: Option<String>,
    /// Mopidy RPC base URL (e.g. `http://mopidy.local:6680`). `None` for non-mopidy devices.
    pub base_url: Option<String>,
}

pub trait DeviceRepository: Send + Sync {
    type Error;

    fn create(&self, device: Device) -> Result<Device, Self::Error>;
    fn get_devices_of_user(&self, user_id: Uuid) -> Result<Vec<Device>, Self::Error>;
    fn delete_by_user_id(&self, user_id: Uuid) -> Result<(), Self::Error>;

    /// Castable devices (Chromecast or Mopidy) the given user is allowed to
    /// see — their own personal devices plus any shared/household device on
    /// the instance.
    fn list_castable_for_user(&self, user_id: Uuid) -> Result<Vec<Device>, Self::Error>;

    fn find_by_chromecast_uuid(&self, chromecast_uuid: &str)
    -> Result<Option<Device>, Self::Error>;

    /// Look up a single device row by its primary id. Real implementors override this.
    fn find_by_id(&self, _id: Uuid) -> Result<Option<Device>, Self::Error> {
        Ok(None)
    }

    /// Delete a single device row by primary id; returns rows removed.
    /// Real implementors override this.
    fn delete_by_id(&self, _id: Uuid) -> Result<usize, Self::Error> {
        Ok(0)
    }

    /// Insert or update a Chromecast row reported by an agent. Lookup is
    /// keyed by `chromecast_uuid`. New rows default to `chromecast_personal`
    /// owned by the agent's user; an admin can later promote `kind` to
    /// `chromecast_shared` from the UI.
    fn upsert_chromecast_from_agent(
        &self,
        chromecast_uuid: &str,
        agent_id: &str,
        owner_user_id: Uuid,
        name: &str,
        ip: Option<&str>,
        last_seen_at: NaiveDateTime,
    ) -> Result<Device, Self::Error>;
}

#[cfg(test)]
mod mopidy_kind_tests {
    use super::kind;

    #[test]
    fn mopidy_kinds_are_mopidy_and_castable_but_not_chromecast() {
        assert!(kind::is_mopidy(kind::MOPIDY_PERSONAL));
        assert!(kind::is_mopidy(kind::MOPIDY_SHARED));
        assert!(!kind::is_mopidy(kind::CHROMECAST_SHARED));

        assert!(kind::is_castable(kind::MOPIDY_SHARED));
        assert!(kind::is_castable(kind::CHROMECAST_PERSONAL));
        assert!(!kind::is_castable(kind::DESKTOP));

        assert!(!kind::is_chromecast(kind::MOPIDY_PERSONAL));
    }
}
