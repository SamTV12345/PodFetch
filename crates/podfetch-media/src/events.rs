//! Events the media library publishes (its published language).
//!
//! Downstream contexts read them with their own types, so the field names
//! and type strings here are a contract: add fields freely, but rename or
//! remove one only with a new event version. The type strings keep the
//! `podcast.` prefix they were stored with.

use podfetch_events::Event;
use serde::{Deserialize, Serialize};

/// An episode file was downloaded and is ready to be played.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpisodeDownloaded {
    pub episode_id: String,
    pub episode_name: String,
    pub podcast_id: String,
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
    pub podcast_id: String,
    pub reason: String,
}

impl Event for EpisodeDownloadFailed {
    const TYPE: &'static str = "podcast.episode_download_failed";

    fn aggregate_id(&self) -> String {
        self.episode_id.clone()
    }
}
