use chrono::NaiveDateTime;

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Device {
    pub id: Option<i32>,
    pub deviceid: String,
    pub kind: String,
    pub name: String,
    pub user_id: i32,
    pub chromecast_uuid: Option<String>,
    pub agent_id: Option<String>,
    pub last_seen_at: Option<NaiveDateTime>,
    pub ip: Option<String>,
}

pub trait DeviceRepository: Send + Sync {
    type Error;

    fn create(&self, device: Device) -> Result<Device, Self::Error>;
    fn get_devices_of_user(&self, user_id: i32) -> Result<Vec<Device>, Self::Error>;
    fn delete_by_user_id(&self, user_id: i32) -> Result<(), Self::Error>;

    /// Chromecast devices the given user is allowed to see — their own
    /// personal devices plus any shared/household devices on the instance.
    fn list_castable_for_user(&self, user_id: i32) -> Result<Vec<Device>, Self::Error>;
}
