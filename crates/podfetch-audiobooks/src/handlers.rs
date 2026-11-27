//! Event handlers of the audiobooks context.

use crate::persistence::user_data::delete_user_data;
use common_infrastructure::db::Database;
use podfetch_events::{EventBus, HandlerError};
use podfetch_identity::events::UserDeleted;

pub const CLEANUP_ON_USER_DELETED: &str = "audiobooks.cleanup_on_user_deleted";

/// Removes the progress and sessions of a deleted user. The tables have no
/// foreign key to the users, so nothing else would.
pub fn subscribe(bus: &EventBus, database: Database) {
    bus.subscribe::<UserDeleted, _, _>(CLEANUP_ON_USER_DELETED, move |event, _| {
        let database = database.clone();
        async move {
            let removed = delete_user_data(&database, event.user_id.as_uuid())
                .map_err(|e| HandlerError::retry(e.to_string()))?;
            tracing::info!(
                "Removed {removed} audiobook progress and session rows of deleted user {}",
                event.username
            );
            Ok(())
        }
    });
}
