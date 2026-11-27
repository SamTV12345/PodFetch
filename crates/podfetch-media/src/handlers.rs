//! Event handlers of the media library.

use crate::auto_download::AutoDownload;
use crate::inbound::PodcastRefreshed;
use crate::sponsorblock::SponsorblockService;
use podfetch_events::{EventBus, HandlerError};
use podfetch_identity::events::UserDeleted;
use std::sync::Arc;

pub const CLEANUP_ON_USER_DELETED: &str = "media.cleanup_on_user_deleted";
pub const AUTO_DOWNLOAD_ON_PODCAST_REFRESHED: &str = "media.auto_download_on_podcast_refreshed";

/// Removes the SponsorBlock preferences of a deleted user.
pub fn subscribe_user_cleanup(bus: &EventBus, sponsorblock: Arc<SponsorblockService>) {
    bus.subscribe::<UserDeleted, _, _>(CLEANUP_ON_USER_DELETED, move |event, _| {
        let sponsorblock = sponsorblock.clone();
        async move {
            sponsorblock
                .delete_preferences(event.user_id)
                .map_err(|error| HandlerError::retry(error.to_string()))?;
            tracing::info!(
                "Removed SponsorBlock preferences of deleted user {}",
                event.username
            );
            Ok(())
        }
    });
}

/// Queues the latest episodes of a refreshed podcast for download. Only
/// queues (see [`crate::auto_download`]); the downloads run afterwards.
pub fn subscribe_auto_download(bus: &EventBus, auto_download: Arc<AutoDownload>) {
    bus.subscribe::<PodcastRefreshed, _, _>(AUTO_DOWNLOAD_ON_PODCAST_REFRESHED, move |event, _| {
        let auto_download = auto_download.clone();
        async move {
            let queued = tokio::task::spawn_blocking(move || {
                auto_download.podcast_refreshed(&event.podcast_id)
            })
            .await
            .map_err(|error| HandlerError::retry(error.to_string()))?
            .map_err(|error| HandlerError::retry(error.to_string()))?;
            if queued > 0 {
                tracing::info!("Queued {queued} episodes for auto-download");
            }
            Ok(())
        }
    });
}
