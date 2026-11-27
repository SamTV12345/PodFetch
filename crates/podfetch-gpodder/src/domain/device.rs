use chrono::NaiveDateTime;
use uuid::Uuid;

/// Discriminator values used in the `kind` column.
/// Values of the `kind` column of the shared `devices` table.
pub mod kind {
    /// gPodder client kinds.
    pub const DESKTOP: &str = "desktop";
    pub const LAPTOP: &str = "laptop";
    pub const MOBILE: &str = "mobile";
    pub const SERVER: &str = "server";
    pub const OTHER: &str = "other";

    /// The kinds the casting context stores in the same table. gPodder
    /// leaves these rows alone and hides them from gPodder clients; a test
    /// in the web crate keeps this list in step with the casting context.
    pub const CASTABLE: [&str; 4] = [
        "chromecast_personal",
        "chromecast_shared",
        "mopidy_personal",
        "mopidy_shared",
    ];

    pub fn is_castable(kind: &str) -> bool {
        CASTABLE.contains(&kind)
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
    /// The user's gPodder devices (cast targets in the same table excluded).
    fn get_devices_of_user(&self, user_id: Uuid) -> Result<Vec<Device>, Self::Error>;
    fn delete_by_user_id(&self, user_id: Uuid) -> Result<(), Self::Error>;
}
