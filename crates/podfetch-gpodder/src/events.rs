//! Events of the gPodder sync context and the ones it reacts to.

use podfetch_events::Event;
use podfetch_kernel::UserId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A gPodder device that shares its subscriptions with the web UI added
/// or removed feeds. The library mirrors this into the user's favorites.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscriptionsChanged {
    pub user_id: UserId,
    pub device: String,
    pub added: Vec<String>,
    pub removed: Vec<String>,
}

impl Event for SubscriptionsChanged {
    const TYPE: &'static str = "gpodder.subscriptions_changed";

    fn aggregate_id(&self) -> String {
        self.user_id.to_string()
    }
}

/// Read side of the library's `library.podcast_favor_changed`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PodcastFavorChanged {
    pub user_id: UserId,
    pub podcast_id: Uuid,
    pub favored: bool,
}

impl Event for PodcastFavorChanged {
    const TYPE: &'static str = "library.podcast_favor_changed";

    fn aggregate_id(&self) -> String {
        self.user_id.to_string()
    }
}
