//! Audiobookshelf-compatible socket.io connection handler.
//!
//! Mirrors upstream `SocketAuthority.js`:
//!  - lives on the DEFAULT namespace `/` (the mobile apps don't talk to
//!    custom namespaces)
//!  - accepts an unauthenticated initial connection
//!  - waits for the client to emit `socket.emit('auth', <token>)` and
//!    validates that token; on success joins the per-user room (room name =
//!    user.id as string) and emits `init` with `{userId, username}`; on
//!    failure emits `auth_failed` with `{message}`
//!
//! The token is the same value the HTTP API uses as Bearer - the user's
//! `users.api_key`, looked up via `AudiobookshelfLoginService`.

use crate::audiobookshelf_api::socket_io::events::{self, EVENT_INIT};
use crate::services::audiobookshelf::login_service::AudiobookshelfLoginService;
use serde_json::Value;
use socketioxide::extract::{Data, SocketRef};
use std::sync::Arc;

pub const EVENT_AUTH: &str = "auth";
pub const EVENT_AUTH_FAILED: &str = "auth_failed";

/// Installs the `auth` event handler on a freshly-connected socket.
/// Called from `startup.rs` for **every** new socket when the
/// audiobookshelf integration is enabled so the mobile apps land in the
/// same default `/` namespace as the rest of PodFetch's socket.io.
pub fn install_auth_handler(socket: &SocketRef, login_service: Arc<AudiobookshelfLoginService>) {
    let svc = login_service.clone();
    socket.on(EVENT_AUTH, move |s: SocketRef, Data::<Value>(token)| {
        let svc = svc.clone();
        async move {
            let token_str = extract_token_value(&token);
            handle_auth(&s, &svc, token_str).await;
        }
    });
}

async fn handle_auth(
    socket: &SocketRef,
    login_service: &AudiobookshelfLoginService,
    token: Option<String>,
) {
    let Some(token) = token.filter(|t| !t.is_empty()) else {
        tracing::warn!(
            "audiobookshelf socket {}: empty auth token",
            socket.id
        );
        emit_auth_failed(socket, "Missing token");
        return;
    };
    let user = match login_service.user_from_token(&token) {
        Ok(Some(user)) => user,
        Ok(None) | Err(_) => {
            tracing::warn!(
                "audiobookshelf socket {}: invalid auth token",
                socket.id
            );
            emit_auth_failed(socket, "Invalid token");
            return;
        }
    };
    socket.join(user.id.to_string());
    let init_payload = events::build_init_payload(&user);
    if let Err(e) = socket.emit(EVENT_INIT.to_string(), &init_payload) {
        tracing::warn!(
            "audiobookshelf socket {}: init emit failed: {e:?}",
            socket.id
        );
    } else {
        tracing::info!(
            "audiobookshelf socket {} authenticated as user {} (id={})",
            socket.id,
            user.username,
            user.id
        );
    }
}

fn emit_auth_failed(socket: &SocketRef, message: &str) {
    let payload = serde_json::json!({ "message": message });
    let _ = socket.emit(EVENT_AUTH_FAILED.to_string(), &payload);
}

/// Accepts the `auth` payload in any of the forms the audiobookshelf
/// clients are known to send it:
///   - bare string: `socket.emit('auth', '<jwt>')`
///   - object with `token` field: `socket.emit('auth', {token: '<jwt>'})`
fn extract_token_value(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Object(map) => map.get("token").and_then(|v| v.as_str()).map(str::to_string),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extract_token_accepts_bare_string() {
        let v = json!("abs_xyz");
        assert_eq!(extract_token_value(&v), Some("abs_xyz".to_string()));
    }

    #[test]
    fn extract_token_accepts_object_with_token_field() {
        let v = json!({ "token": "abs_xyz" });
        assert_eq!(extract_token_value(&v), Some("abs_xyz".to_string()));
    }

    #[test]
    fn extract_token_rejects_other_shapes() {
        assert_eq!(extract_token_value(&json!(null)), None);
        assert_eq!(extract_token_value(&json!(42)), None);
        assert_eq!(extract_token_value(&json!({})), None);
        assert_eq!(extract_token_value(&json!({"other": "x"})), None);
    }

    #[test]
    fn event_names_are_audiobookshelf_compatible() {
        // Pinned wire names — if any of these change, the mobile apps stop working.
        assert_eq!(EVENT_AUTH, "auth");
        assert_eq!(EVENT_AUTH_FAILED, "auth_failed");
        assert_eq!(EVENT_INIT, "init");
    }
}
