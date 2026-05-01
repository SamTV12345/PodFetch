//! Local Chromecast driver wrapping the synchronous [`rust_cast`] crate.
//!
//! Exposes async `play` / `control` methods. Each call runs the blocking
//! CAST handshake via [`tokio::task::spawn_blocking`] so the agent's
//! tokio runtime stays responsive.
//!
//! Scope of this module today: the **Play** path is real — connect, launch
//! the Default Media Receiver, load the requested URL, return our own
//! correlated [`CastSessionId`].
//!
//! Pause / Resume / Stop / Seek are not yet implemented. Doing them
//! correctly requires a long-lived CAST connection per session plus a
//! receiver loop that drains MediaStatus updates back into the orchestrator
//! — that's its own sub-phase. Until then, [`LocalCastDriver::control`]
//! returns [`CastDriveError::NotImplemented`] with a clear message.

use podfetch_cast::{CastMedia, CastSessionId, CastTarget, ControlCmd};
use rust_cast::CastDevice;
use rust_cast::channels::media::{Media, StreamType};
use rust_cast::channels::receiver::CastDeviceApp;
use std::collections::HashMap;
use std::sync::{Mutex, Once};
use thiserror::Error;
use tracing::{info, warn};

/// Receiver id used to address the platform-level receiver. Hard-coded by
/// the protocol — it's not the launched-app's transport id.
const RECEIVER_DESTINATION: &str = "receiver-0";

/// rustls 0.23 requires a crypto provider to be installed before any TLS
/// op. Workspaces that pull in both `aws-lc-rs` and `ring` (we do — via
/// reqwest + rust_cast) make auto-selection ambiguous, so we install one
/// explicitly the first time the driver is used. Idempotent via Once.
static INSTALL_CRYPTO: Once = Once::new();

fn ensure_crypto_provider() {
    INSTALL_CRYPTO.call_once(|| {
        // ignore the result: a provider may already be installed when this
        // process is also acting as a server.
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

/// Per-session state cached by the driver. Keeps enough information to
/// later issue Pause/Resume/Stop/Seek once that path is implemented.
#[derive(Debug, Clone)]
pub struct StoredSession {
    pub host: String,
    pub port: u16,
    pub app_transport_id: String,
    pub app_session_id: String,
    pub media_session_id: i32,
}

#[derive(Debug, Error)]
pub enum CastDriveError {
    #[error("device {host}:{port} unreachable: {reason}")]
    Connect { host: String, port: u16, reason: String },
    #[error("CAST receiver rejected request: {0}")]
    Receiver(String),
    #[error("not implemented: {0}")]
    NotImplemented(&'static str),
    #[error("internal: {0}")]
    Internal(String),
}

#[derive(Default)]
pub struct LocalCastDriver {
    sessions: Mutex<HashMap<CastSessionId, StoredSession>>,
}

impl LocalCastDriver {
    pub fn new() -> Self {
        Self::default()
    }

    /// Launch the Default Media Receiver app on `target` and load `media`.
    /// On success returns a fresh CastSessionId — that id is what we hand
    /// to the server so subsequent control commands can find the session.
    pub async fn play(
        &self,
        target: &CastTarget,
        media: &CastMedia,
    ) -> Result<CastSessionId, CastDriveError> {
        let host = target.ip.to_string();
        let port = target.port;
        let url = media.url.clone();
        let mime = media.mime.clone();
        let duration = media.duration_secs.map(|d| d as f32);

        ensure_crypto_provider();
        let stored = tokio::task::spawn_blocking(move || -> Result<StoredSession, CastDriveError> {
            let cast = CastDevice::connect(host.clone(), port)
                .map_err(|err| CastDriveError::Connect {
                    host: host.clone(),
                    port,
                    reason: err.to_string(),
                })?;

            cast.connection
                .connect(RECEIVER_DESTINATION)
                .map_err(|e| CastDriveError::Internal(format!("connection.connect: {e}")))?;
            cast.heartbeat
                .ping()
                .map_err(|e| CastDriveError::Internal(format!("heartbeat.ping: {e}")))?;

            let app = cast
                .receiver
                .launch_app(&CastDeviceApp::DefaultMediaReceiver)
                .map_err(|e| CastDriveError::Receiver(format!("launch_app: {e}")))?;

            cast.connection
                .connect(app.transport_id.as_str())
                .map_err(|e| CastDriveError::Internal(format!("app connect: {e}")))?;

            let to_load = Media {
                content_id: url,
                stream_type: StreamType::Buffered,
                content_type: mime,
                metadata: None,
                duration,
            };
            let status = cast
                .media
                .load(
                    app.transport_id.as_str(),
                    app.session_id.as_str(),
                    &to_load,
                )
                .map_err(|e| CastDriveError::Receiver(format!("media.load: {e}")))?;

            let media_session_id = status
                .entries
                .first()
                .map(|entry| entry.media_session_id)
                .ok_or_else(|| {
                    CastDriveError::Receiver("CAST status returned no entries".into())
                })?;

            Ok(StoredSession {
                host,
                port,
                app_transport_id: app.transport_id,
                app_session_id: app.session_id,
                media_session_id,
            })
        })
        .await
        .map_err(|e| CastDriveError::Internal(format!("blocking task panic: {e}")))??;

        let session_id = CastSessionId::new();
        info!(
            session = %session_id.0,
            transport = %stored.app_transport_id,
            media_session = stored.media_session_id,
            "CAST play succeeded"
        );
        self.sessions
            .lock()
            .expect("driver session lock poisoned")
            .insert(session_id.clone(), stored);
        Ok(session_id)
    }

    /// Pause / Resume / Stop / Seek — pending design of the per-session
    /// long-lived connection. See module docs.
    pub async fn control(
        &self,
        session_id: &CastSessionId,
        _cmd: &ControlCmd,
    ) -> Result<(), CastDriveError> {
        // Even though we don't drive the session, surface a clearer error
        // when it's an entirely unknown id vs. just unimplemented.
        let known = self
            .sessions
            .lock()
            .expect("driver session lock poisoned")
            .contains_key(session_id);
        if !known {
            warn!(session = %session_id.0, "control on unknown session");
        }
        Err(CastDriveError::NotImplemented(
            "agent control commands are pending a long-lived CAST connection design",
        ))
    }

    /// True iff the driver currently tracks the given session id. Mostly
    /// useful for tests.
    pub fn knows_session(&self, session_id: &CastSessionId) -> bool {
        self.sessions
            .lock()
            .expect("driver session lock poisoned")
            .contains_key(session_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use podfetch_cast::CastDeviceUuid;
    use std::net::{IpAddr, Ipv4Addr};

    fn unreachable_target() -> CastTarget {
        CastTarget {
            uuid: CastDeviceUuid("test".into()),
            // 240.0.0.1 is reserved (RFC 1112) and won't route, so the
            // CAST connect attempt fails fast without touching real
            // network or hardware.
            ip: IpAddr::V4(Ipv4Addr::new(240, 0, 0, 1)),
            port: 8009,
        }
    }

    fn media() -> CastMedia {
        CastMedia {
            url: "https://example.com/audio.mp3".into(),
            mime: "audio/mpeg".into(),
            title: "Episode 1".into(),
            artwork_url: None,
            duration_secs: Some(60.0),
            episode_id: Some(7),
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn play_against_unreachable_device_returns_connect_error() {
        let driver = LocalCastDriver::new();
        let result = driver.play(&unreachable_target(), &media()).await;
        match result {
            Err(CastDriveError::Connect { .. }) => {}
            other => panic!("expected Connect error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn control_on_unknown_session_is_not_implemented() {
        let driver = LocalCastDriver::new();
        let result = driver
            .control(&CastSessionId("ghost".into()), &ControlCmd::Pause)
            .await;
        match result {
            Err(CastDriveError::NotImplemented(_)) => {}
            other => panic!("expected NotImplemented, got {other:?}"),
        }
    }

    #[test]
    fn unknown_session_is_not_tracked() {
        let driver = LocalCastDriver::new();
        assert!(!driver.knows_session(&CastSessionId("nope".into())));
    }
}
