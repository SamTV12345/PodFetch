//! Event handlers of the library context.

use crate::domain::favorite::FavoriteRepository;
use crate::events::SubscriptionsChanged;
use common_infrastructure::error::CustomError;
use podfetch_events::{EventBus, HandlerError};
use podfetch_identity::events::UserDeleted;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub const CLEANUP_ON_USER_DELETED: &str = "library.cleanup_on_user_deleted";
pub const FAVORITES_FROM_GPODDER: &str = "library.favorites_from_gpodder";

/// Port to the catalog: which podcasts the given feed URLs belong to.
/// Feeds that are not in the catalog are left out.
pub trait FeedResolver: Send + Sync {
    fn podcasts_by_feed(&self, feeds: &[String]) -> Result<HashMap<String, Uuid>, CustomError>;
}

fn retry(error: CustomError) -> HandlerError {
    HandlerError::retry(error.to_string())
}

/// Removes the favorites of a deleted user.
pub fn subscribe_user_cleanup(
    bus: &EventBus,
    favorites: Arc<dyn FavoriteRepository<Error = CustomError>>,
) {
    bus.subscribe::<UserDeleted, _, _>(CLEANUP_ON_USER_DELETED, move |event, _| {
        let favorites = favorites.clone();
        async move {
            favorites
                .delete_by_user_id(event.user_id.as_uuid())
                .map_err(retry)?;
            tracing::info!("Removed favorites of deleted user {}", event.username);
            Ok(())
        }
    });
}

/// Mirrors subscriptions of gPodder devices that share them with the web
/// UI into the user's favorite podcasts. Writes the favorites directly, so
/// no `PodcastFavorChanged` is published back to gPodder sync.
pub fn subscribe_favorite_sync(
    bus: &EventBus,
    favorites: Arc<dyn FavoriteRepository<Error = CustomError>>,
    feeds: Arc<dyn FeedResolver>,
) {
    bus.subscribe::<SubscriptionsChanged, _, _>(FAVORITES_FROM_GPODDER, move |event, _| {
        let favorites = favorites.clone();
        let feeds = feeds.clone();
        async move {
            let user_id = event.user_id.as_uuid();
            let all: Vec<String> = event.added.iter().chain(&event.removed).cloned().collect();
            let podcasts = feeds.podcasts_by_feed(&all).map_err(retry)?;
            let changes = event
                .added
                .iter()
                .map(|feed| (feed, true))
                .chain(event.removed.iter().map(|feed| (feed, false)));
            for (feed, favored) in changes {
                if let Some(&podcast_id) = podcasts.get(feed) {
                    favorites
                        .update_podcast_favor(podcast_id, favored, user_id)
                        .map_err(retry)?;
                }
            }
            Ok(())
        }
    });
}
