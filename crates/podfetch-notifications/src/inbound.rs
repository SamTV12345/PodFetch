//! Events this context reacts to, read in its own terms.
//!
//! The podcast context publishes these; Notifications only depends on the
//! event contract (type name and the JSON fields below), never on the
//! publisher's code. Unknown fields are ignored, so the publisher may add
//! fields without breaking this reader.

use podfetch_events::Event;
use serde::{Deserialize, Serialize};

/// An episode file was downloaded and is ready to be played.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpisodeDownloaded {
    pub episode_id: String,
    pub episode_name: String,
    pub podcast_name: String,
}

impl Event for EpisodeDownloaded {
    const TYPE: &'static str = "podcast.episode_downloaded";

    fn aggregate_id(&self) -> String {
        self.episode_id.clone()
    }
}

/// Downloading an episode failed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpisodeDownloadFailed {
    pub episode_id: String,
    pub episode_name: String,
    pub reason: String,
}

impl Event for EpisodeDownloadFailed {
    const TYPE: &'static str = "podcast.episode_download_failed";

    fn aggregate_id(&self) -> String {
        self.episode_id.clone()
    }
}
