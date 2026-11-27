//! Events of the library context and the ones it reacts to.

use podfetch_events::Event;
use podfetch_kernel::UserId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A user favored or unfavored a podcast. gPodder sync mirrors this into
/// the subscriptions of the devices that share them with the web UI.
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

/// Read side of gPodder sync's `gpodder.subscriptions_changed`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscriptionsChanged {
    pub user_id: UserId,
    pub added: Vec<String>,
    pub removed: Vec<String>,
}

impl Event for SubscriptionsChanged {
    const TYPE: &'static str = "gpodder.subscriptions_changed";

    fn aggregate_id(&self) -> String {
        self.user_id.to_string()
    }
}
