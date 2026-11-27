//! Events the podcast context publishes (its published language).
//!
//! Downstream contexts read them with their own types, so the field names
//! and type strings here are a contract: add fields freely, but rename or
//! remove one only with a new event version.

use podfetch_events::{Event, EventBus};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, PoisonError, RwLock};

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

/// The bus the download code and the static podcast service publish to
/// (also the library's `PodcastFavorChanged` from `update_favor_podcast`).
///
/// Transitional: the download pipeline is still a set of associated
/// functions without injected dependencies. Once the podcast context gets
/// its own services (migration step 4), the bus is passed in and this goes
/// away.
static BUS: RwLock<Option<Arc<EventBus>>> = RwLock::new(None);

/// Makes `bus` the one the podcast context publishes to. Called while the
/// application state is built; a later call replaces the earlier bus.
pub(crate) fn install(bus: Arc<EventBus>) {
    *BUS.write().unwrap_or_else(PoisonError::into_inner) = Some(bus);
}

/// Publishes `event`. A failure is logged and does not fail the download
/// that caused it.
pub(crate) fn publish<E: Event>(event: &E) {
    let bus = BUS.read().unwrap_or_else(PoisonError::into_inner).clone();
    let Some(bus) = bus else {
        tracing::error!("No event bus installed, dropping {}", E::TYPE);
        return;
    };
    if let Err(err) = bus.publish(event) {
        tracing::error!("Could not publish {}: {err}", E::TYPE);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use podfetch_notifications::inbound;

    /// The notifications context reads these events with its own types;
    /// this pins the contract between both sides.
    #[test]
    fn notifications_can_read_episode_downloaded() {
        let published = EpisodeDownloaded {
            episode_id: "e1".into(),
            episode_name: "Episode 1".into(),
            podcast_id: "p1".into(),
            podcast_name: "Podcast".into(),
        };
        let read: inbound::EpisodeDownloaded =
            serde_json::from_value(serde_json::to_value(&published).unwrap()).unwrap();
        assert_eq!(EpisodeDownloaded::TYPE, inbound::EpisodeDownloaded::TYPE);
        assert_eq!(
            EpisodeDownloaded::VERSION,
            inbound::EpisodeDownloaded::VERSION
        );
        assert_eq!(read.episode_id, "e1");
        assert_eq!(read.episode_name, "Episode 1");
        assert_eq!(read.podcast_name, "Podcast");
    }

    #[test]
    fn notifications_can_read_episode_download_failed() {
        let published = EpisodeDownloadFailed {
            episode_id: "e1".into(),
            episode_name: "Episode 1".into(),
            podcast_id: "p1".into(),
            reason: "timeout".into(),
        };
        let read: inbound::EpisodeDownloadFailed =
            serde_json::from_value(serde_json::to_value(&published).unwrap()).unwrap();
        assert_eq!(
            EpisodeDownloadFailed::TYPE,
            inbound::EpisodeDownloadFailed::TYPE
        );
        assert_eq!(
            EpisodeDownloadFailed::VERSION,
            inbound::EpisodeDownloadFailed::VERSION
        );
        assert_eq!(read.reason, "timeout");
    }

    /// The media library reads `podcast.refreshed` with its own type.
    #[test]
    fn media_can_read_podcast_refreshed() {
        let published = PodcastRefreshed {
            podcast_id: "p1".into(),
            podcast_name: "Podcast".into(),
        };
        let read: podfetch_media::inbound::PodcastRefreshed =
            serde_json::from_value(serde_json::to_value(&published).unwrap()).unwrap();
        assert_eq!(
            PodcastRefreshed::TYPE,
            podfetch_media::inbound::PodcastRefreshed::TYPE
        );
        assert_eq!(
            PodcastRefreshed::VERSION,
            podfetch_media::inbound::PodcastRefreshed::VERSION
        );
        assert_eq!(read.podcast_id, "p1");
    }

    /// Full path: the download code publishes, the processor delivers and
    /// the notifications context stores the in-app notifications.
    #[tokio::test]
    #[serial_test::serial]
    async fn download_events_end_up_as_in_app_notifications() {
        let _server = crate::test_support::tests::handle_test_startup().await;
        let state = crate::test_support::tests::app_state();
        let processor = state.event_bus.spawn_processor();

        publish(&EpisodeDownloaded {
            episode_id: "episode-ok".into(),
            episode_name: "Fresh Episode".into(),
            podcast_id: "p1".into(),
            podcast_name: "Podcast".into(),
        });
        publish(&EpisodeDownloadFailed {
            episode_id: "episode-broken".into(),
            episode_name: "Broken Episode".into(),
            podcast_id: "p1".into(),
            reason: "404".into(),
        });

        let mut unread = Vec::new();
        for _ in 0..100 {
            unread = state
                .notification_service
                .get_unread_notifications()
                .unwrap();
            if unread.len() >= 2 {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
        processor.stop(std::time::Duration::from_secs(1)).await;

        let find = |kind: &str| {
            unread
                .iter()
                .find(|n| n.type_of_message == kind)
                .unwrap_or_else(|| panic!("no {kind} notification in {unread:?}"))
        };
        assert_eq!(find("Download").message, "Fresh Episode");
        assert_eq!(find("DownloadFailed").message, "Broken Episode (404)");
        assert_eq!(unread.len(), 2);
    }
}
