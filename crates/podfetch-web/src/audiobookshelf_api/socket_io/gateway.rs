//! Registers a socket.io namespace handler that authenticates audiobookshelf
//! mobile-app connections, joins them into the per-user room used for
//! targeted broadcasts, and emits the `init` payload immediately.

use crate::audiobookshelf_api::socket_io::events::{self, EVENT_INIT};
use crate::services::audiobookshelf::login_service::AudiobookshelfLoginService;
use socketioxide::SocketIo;
use socketioxide::extract::SocketRef;
use std::sync::Arc;

/// Wires the audiobookshelf socket.io handler into the existing socket.io
/// instance. Idempotent at most once per `SocketIo` (callers should only
/// invoke this during server startup).
pub fn register(io: &SocketIo, login_service: Arc<AudiobookshelfLoginService>) {
    let login_service = login_service.clone();
    io.ns("/audiobookshelf", move |socket: SocketRef| {
        let login_service = login_service.clone();
        async move {
            let Some(token) = extract_token(&socket) else {
                tracing::warn!(
                    "audiobookshelf socket: rejecting connection {} - missing token",
                    socket.id
                );
                let _ = socket.disconnect();
                return;
            };
            let user = match login_service.user_from_token(&token) {
                Ok(Some(user)) => user,
                Ok(None) | Err(_) => {
                    tracing::warn!(
                        "audiobookshelf socket: rejecting connection {} - invalid token",
                        socket.id
                    );
                    let _ = socket.disconnect();
                    return;
                }
            };
            let room = user.id.to_string();
            socket.join(room.clone());
            let init_payload = events::build_init_payload(&user);
            if let Err(e) = socket.emit(EVENT_INIT.to_string(), &init_payload) {
                tracing::warn!(
                    "audiobookshelf socket: init emit failed for {}: {e:?}",
                    socket.id
                );
            }
            tracing::info!(
                "audiobookshelf socket connected: id={} user={} room={room}",
                socket.id,
                user.username
            );
        }
    });
}

fn extract_token(socket: &SocketRef) -> Option<String> {
    let parts = socket.req_parts();
    let query = parts.uri.query().unwrap_or("");
    for pair in query.split('&') {
        let candidate = pair
            .strip_prefix("token=")
            .or_else(|| pair.strip_prefix("apiKey="));
        if let Some(value) = candidate {
            let decoded = urlencoding::decode(value).unwrap_or_default();
            if !decoded.is_empty() {
                return Some(decoded.into_owned());
            }
        }
    }
    None
}
