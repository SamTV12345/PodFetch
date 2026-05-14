//! Thin wrapper that emits typed audiobookshelf events into the global
//! socket.io instance, scoped by per-user rooms.
//!
//! Per-user routing follows audiobookshelf's `clientEmitter(userId, ...)`
//! semantics: every connected socket for a user has joined the room named by
//! the user's id (as a string). Emissions target that room so all of a user's
//! devices receive the same event.

use crate::audiobookshelf_api::socket_io::events::{
    self, EVENT_LIBRARY_ADDED, EVENT_LIBRARY_REMOVED, EVENT_LIBRARY_UPDATED,
    EVENT_USER_ITEM_PROGRESS_UPDATED, EVENT_USER_UPDATED,
};
use crate::server::SOCKET_IO_LAYER;
use podfetch_domain::audiobookshelf::library::Library;
use podfetch_domain::audiobookshelf::media_progress::MediaProgress;
use podfetch_domain::user::User;

pub fn user_room(user_id: i32) -> String {
    user_id.to_string()
}

/// Emit `user_item_progress_updated` to all of the given user's sockets.
/// No-op when the socket.io layer is not yet initialised (tests, startup).
pub fn emit_progress_updated(
    user_id: i32,
    progress: &MediaProgress,
    session_id: &str,
    device_description: &str,
) {
    let payload = events::build_progress_updated_payload(progress, session_id, device_description);
    emit_to_user_room(user_id, EVENT_USER_ITEM_PROGRESS_UPDATED, payload);
}

pub fn emit_user_updated(user: &User, media_progress: &[MediaProgress]) {
    let payload = events::build_user_updated_payload(user, media_progress);
    emit_to_user_room(user.id, EVENT_USER_UPDATED, payload);
}

pub fn emit_library_added(library: &Library) {
    let payload = events::build_library_payload(library);
    emit_broadcast(EVENT_LIBRARY_ADDED, payload);
}

pub fn emit_library_updated(library: &Library) {
    let payload = events::build_library_payload(library);
    emit_broadcast(EVENT_LIBRARY_UPDATED, payload);
}

pub fn emit_library_removed(library: &Library) {
    let payload = events::build_library_payload(library);
    emit_broadcast(EVENT_LIBRARY_REMOVED, payload);
}

pub fn emit_item_event(event: &str, payload: serde_json::Value) {
    emit_broadcast(event, payload);
}

fn emit_to_user_room(user_id: i32, event: &str, payload: serde_json::Value) {
    let Some(io) = SOCKET_IO_LAYER.get() else {
        tracing::trace!(
            "audiobookshelf socket.io not initialised; dropping event {event} for user {user_id}"
        );
        return;
    };
    let room = user_room(user_id);
    let event = event.to_string();
    let io_handle = io.clone();
    tokio::spawn(async move {
        if let Err(e) = io_handle
            .to(room.clone())
            .emit(event.clone(), &payload)
            .await
        {
            tracing::warn!("audiobookshelf socket.io emit({event}) to {room} failed: {e:?}");
        }
    });
}

fn emit_broadcast(event: &str, payload: serde_json::Value) {
    let Some(io) = SOCKET_IO_LAYER.get() else {
        tracing::trace!("audiobookshelf socket.io not initialised; dropping broadcast {event}");
        return;
    };
    let event = event.to_string();
    let io_handle = io.clone();
    tokio::spawn(async move {
        if let Err(e) = io_handle.emit(event.clone(), &payload).await {
            tracing::warn!("audiobookshelf socket.io broadcast({event}) failed: {e:?}");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_room_is_string_id_per_upstream_clientemitter() {
        assert_eq!(user_room(42), "42".to_string());
        assert_eq!(user_room(0), "0".to_string());
    }

    #[test]
    fn emit_progress_no_op_when_layer_unset_does_not_panic() {
        // Outside startup, SOCKET_IO_LAYER may be unset; we must degrade
        // gracefully rather than crash the request that triggered it.
        let now = chrono::Utc::now().naive_utc();
        let progress = MediaProgress {
            id: "abc".to_string(),
            user_id: 1,
            library_item_id: "li_pod_1".to_string(),
            episode_id: Some("ep_1".to_string()),
            media_type: "podcast".to_string(),
            duration: 10.0,
            current_time: 1.0,
            progress: 0.1,
            is_finished: false,
            hide_from_continue_listening: false,
            last_update: now,
            started_at: now,
            finished_at: None,
        };
        // Should not panic even when SOCKET_IO_LAYER may or may not be initialised.
        emit_progress_updated(1, &progress, "play_x", "Test Device");
    }
}
