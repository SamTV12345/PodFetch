//! Events of other contexts the media library reacts to, read with its own
//! types (tolerant reader: only the fields it needs, unknown ones ignored).

use podfetch_events::Event;
use serde::{Deserialize, Serialize};

/// The catalog refreshed a podcast's episodes from its feed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PodcastRefreshed {
    pub podcast_id: String,
}

impl Event for PodcastRefreshed {
    const TYPE: &'static str = "podcast.refreshed";

    fn aggregate_id(&self) -> String {
        self.podcast_id.clone()
    }
}
