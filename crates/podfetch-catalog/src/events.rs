//! Events the catalog publishes (its published language).
//!
//! Downstream contexts read them with their own types, so the field names
//! and type strings here are a contract: add fields freely, but rename or
//! remove one only with a new event version.

use podfetch_events::Event;
use serde::{Deserialize, Serialize};

/// The catalog refreshed a podcast's episodes from its feed (after adding
/// it, on every poll and on a manual refresh).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PodcastRefreshed {
    pub podcast_id: String,
    pub podcast_name: String,
}

impl Event for PodcastRefreshed {
    const TYPE: &'static str = "podcast.refreshed";

    fn aggregate_id(&self) -> String {
        self.podcast_id.clone()
    }
}

/// A podcast left the catalog. The other contexts removed what they kept
/// about it before (the web layer's `podcast_removal`); this is for
/// reactions that may happen afterwards.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PodcastRemoved {
    pub podcast_id: String,
    pub podcast_name: String,
    pub rssfeed: String,
}

impl Event for PodcastRemoved {
    const TYPE: &'static str = "podcast.removed";

    fn aggregate_id(&self) -> String {
        self.podcast_id.clone()
    }
}
