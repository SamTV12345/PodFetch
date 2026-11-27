//! Event handlers of the listening context.

use crate::domain::episode::EpisodeRepository;
use crate::persistence::episode::DieselEpisodeRepository;
use crate::services::listening_event::ListeningEventService;
use podfetch_events::{EventBus, HandlerError};
use podfetch_identity::events::UserDeleted;
use std::sync::Arc;

pub const CLEANUP_ON_USER_DELETED: &str = "listening.cleanup_on_user_deleted";

/// Registers the handlers of this context.
pub fn subscribe(
    bus: &EventBus,
    actions: Arc<DieselEpisodeRepository>,
    listening_events: Arc<ListeningEventService>,
) {
    bus.subscribe::<UserDeleted, _, _>(CLEANUP_ON_USER_DELETED, move |event, _| {
        let actions = actions.clone();
        let listening_events = listening_events.clone();
        async move {
            let user_id = event.user_id.as_uuid();
            let events = listening_events
                .delete_by_user_id(user_id)
                .map_err(|e| HandlerError::retry(e.to_string()))?;
            let removed = actions
                .delete_by_user(user_id)
                .map_err(|e| HandlerError::retry(e.to_string()))?;
            tracing::info!(
                "Removed {removed} episode actions and {events} listening events of deleted user {}",
                event.username
            );
            Ok(())
        }
    });
}
