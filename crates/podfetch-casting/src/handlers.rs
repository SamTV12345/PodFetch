//! Event handlers of the casting context.

use crate::dto::ServerCastOrchestrator;
use podfetch_events::{EventBus, HandlerError};
use podfetch_identity::events::UserDeleted;
use std::sync::Arc;

pub const CLEANUP_ON_USER_DELETED: &str = "casting.cleanup_on_user_deleted";

/// Registers the handlers of this context.
/// Takes the concrete orchestrator: `CastDriver` futures are only known to
/// be `Send` for a concrete driver.
pub fn subscribe(bus: &EventBus, orchestrator: Arc<ServerCastOrchestrator>) {
    bus.subscribe::<UserDeleted, _, _>(CLEANUP_ON_USER_DELETED, move |event, _| {
        let orchestrator = orchestrator.clone();
        async move {
            let user_id = event.user_id.as_uuid();
            let ended = orchestrator.end_sessions_of(user_id).await;
            let removed = orchestrator
                .devices()
                .delete_owned_by(user_id)
                .map_err(|e| HandlerError::retry(e.to_string()))?;
            tracing::info!(
                "Removed {removed} cast devices and ended {} sessions of deleted user {}",
                ended.len(),
                event.username
            );
            Ok(())
        }
    });
}
