//! Event handlers of the gPodder sync context.

use crate::domain::device::DeviceRepository;
use crate::domain::device_sync_group::DeviceSyncGroupRepository;
use crate::domain::gpodder_setting::GpodderSettingRepository;
use crate::domain::subscription::SubscriptionRepository;
use crate::dto::subscription::SubscriptionUpdateRequest;
use crate::events::PodcastFavorChanged;
use crate::services::subscription::SubscriptionService;
use common_infrastructure::error::CustomError;
use common_infrastructure::runtime::DEFAULT_DEVICE;
use podfetch_events::{EventBus, HandlerError};
use podfetch_identity::events::UserDeleted;
use std::sync::Arc;
use uuid::Uuid;

pub const CLEANUP_ON_USER_DELETED: &str = "gpodder.cleanup_on_user_deleted";
pub const SUBSCRIPTIONS_FROM_FAVORITES: &str = "gpodder.subscriptions_from_favorites";

/// Port to the catalog: the feed URL gPodder clients know a podcast by.
pub trait PodcastFeeds: Send + Sync {
    fn feed_of(&self, podcast_id: Uuid) -> Result<Option<String>, CustomError>;
}

/// The repositories this context owns, for the user cleanup.
pub struct UserData {
    pub devices: Arc<dyn DeviceRepository<Error = CustomError>>,
    pub subscriptions: Arc<dyn SubscriptionRepository<Error = CustomError>>,
    pub sync_groups: Arc<dyn DeviceSyncGroupRepository<Error = CustomError>>,
    pub settings: Arc<dyn GpodderSettingRepository<Error = CustomError>>,
}

fn retry(error: CustomError) -> HandlerError {
    HandlerError::retry(error.to_string())
}

/// Removes the user's gPodder devices, subscriptions, sync groups and
/// settings. Cast devices in the shared table belong to the casting context.
pub fn subscribe_user_cleanup(bus: &EventBus, data: Arc<UserData>) {
    bus.subscribe::<UserDeleted, _, _>(CLEANUP_ON_USER_DELETED, move |event, _| {
        let data = data.clone();
        async move {
            let user_id = event.user_id.as_uuid();
            data.subscriptions
                .delete_by_user_id(user_id)
                .map_err(retry)?;
            data.devices.delete_by_user_id(user_id).map_err(retry)?;
            data.sync_groups
                .replace_all(user_id, Vec::new())
                .map_err(retry)?;
            data.settings.delete_by_user_id(user_id).map_err(retry)?;
            tracing::info!("Removed gPodder data of deleted user {}", event.username);
            Ok(())
        }
    });
}

/// Mirrors web favorites into the subscriptions of every device that shares
/// its subscriptions with the web UI. Only subscribed when the gPodder
/// integration is enabled.
pub fn subscribe_favorite_sync(
    bus: &EventBus,
    subscriptions: Arc<SubscriptionService>,
    feeds: Arc<dyn PodcastFeeds>,
) {
    bus.subscribe::<PodcastFavorChanged, _, _>(SUBSCRIPTIONS_FROM_FAVORITES, move |event, _| {
        let subscriptions = subscriptions.clone();
        let feeds = feeds.clone();
        async move {
            // The podcast may have been removed since; nothing to mirror.
            let Some(feed) = feeds.feed_of(event.podcast_id).map_err(retry)? else {
                return Ok(());
            };
            let (add, remove) = if event.favored {
                (vec![feed], vec![])
            } else {
                (vec![], vec![feed])
            };
            // Written as the web UI device, so it does not publish
            // `SubscriptionsChanged` back to the library.
            subscriptions
                .update_subscriptions(
                    DEFAULT_DEVICE,
                    event.user_id.as_uuid(),
                    SubscriptionUpdateRequest { add, remove },
                )
                .map(|_| ())
                .map_err(retry)
        }
    });
}
