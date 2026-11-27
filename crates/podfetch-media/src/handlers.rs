//! Event handlers of the media library.

use crate::sponsorblock::SponsorblockService;
use podfetch_events::{EventBus, HandlerError};
use podfetch_identity::events::UserDeleted;
use std::sync::Arc;

pub const CLEANUP_ON_USER_DELETED: &str = "media.cleanup_on_user_deleted";

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
