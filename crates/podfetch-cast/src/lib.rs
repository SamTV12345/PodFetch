//! Chromecast driver abstraction for PodFetch.
//!
//! This crate defines the [`CastDriver`] trait — the contract a backend must
//! satisfy to discover and control Chromecast devices — together with the
//! value types exchanged with callers (orchestrator, agent forwarder, etc.).
//!
//! The actual Chromecast protocol implementation lives behind this trait and
//! is selected per-deployment (local mDNS+CAST in the server, or proxied via
//! the agent websocket from a remote PodFetch instance).

use std::net::IpAddr;

use chrono::{DateTime, Utc};
use futures::stream::BoxStream;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Stable identifier for a discovered Chromecast device.
///
/// Wraps the CAST UUID reported in mDNS TXT records (`id=`) so call sites
/// can't accidentally mix it with other string IDs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CastDeviceUuid(pub String);

impl From<String> for CastDeviceUuid {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl AsRef<str> for CastDeviceUuid {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Identifier for an active CAST media session.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CastSessionId(pub String);

impl CastSessionId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl Default for CastSessionId {
    fn default() -> Self {
        Self::new()
    }
}

/// A device returned by discovery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveredCastDevice {
    pub uuid: CastDeviceUuid,
    pub friendly_name: String,
    pub model: Option<String>,
    pub ip: Option<IpAddr>,
    pub port: u16,
}

/// Where a play/control command should be routed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CastTarget {
    pub uuid: CastDeviceUuid,
    pub ip: IpAddr,
    pub port: u16,
}

/// Media payload to load on the receiver.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CastMedia {
    /// URL the Chromecast will fetch the audio bytes from. Must be reachable
    /// from the Chromecast's network.
    pub url: String,
    /// e.g. `audio/mpeg`, `audio/aac`.
    pub mime: String,
    pub title: String,
    pub artwork_url: Option<String>,
    pub duration_secs: Option<f64>,
    /// Originating PodFetch episode id, used by the orchestrator to wire
    /// status updates back into watchtime tracking.
    pub episode_id: Option<i32>,
}

/// Playback state reported by the receiver.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CastState {
    Idle,
    Buffering,
    Playing,
    Paused,
    Stopped,
}

/// A single status snapshot from the receiver. Status streams emit one of
/// these whenever the device's state, position, or volume changes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CastStatus {
    pub session_id: CastSessionId,
    pub state: CastState,
    pub position_secs: f64,
    pub volume: f32,
    pub at: DateTime<Utc>,
}

/// Control commands the orchestrator can issue against an active session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "cmd")]
pub enum ControlCmd {
    Pause,
    Resume,
    Stop,
    Seek { position_secs: f64 },
    SetVolume { volume: f32 },
}

/// Errors a `CastDriver` operation can fail with.
#[derive(Debug, Error)]
pub enum CastError {
    #[error("device {0:?} not found")]
    DeviceNotFound(CastDeviceUuid),
    #[error("session {0:?} not found")]
    SessionNotFound(CastSessionId),
    #[error("discovery failed: {0}")]
    Discovery(String),
    #[error("transport error: {0}")]
    Transport(String),
    #[error("receiver rejected request: {0}")]
    Receiver(String),
    #[error("not implemented")]
    NotImplemented,
}

/// Backend-agnostic Chromecast control surface.
///
/// Implementations:
/// - **Local** — speaks CAST directly over the LAN (server or agent).
/// - **Agent-routed** — forwards each call as a message over the agent
///   websocket so the remote agent's local driver executes it.
#[allow(async_fn_in_trait)]
pub trait CastDriver: Send + Sync {
    async fn discover(&self) -> Result<Vec<DiscoveredCastDevice>, CastError>;

    async fn play(
        &self,
        target: &CastTarget,
        media: &CastMedia,
    ) -> Result<CastSessionId, CastError>;

    async fn control(&self, session: &CastSessionId, cmd: &ControlCmd) -> Result<(), CastError>;

    async fn status_snapshot(&self, session: &CastSessionId) -> Result<CastStatus, CastError>;

    /// Stream of incremental status updates for an active session. Ends when
    /// the session terminates (stop, error, or device gone).
    fn status_stream(&self, session: &CastSessionId) -> BoxStream<'static, CastStatus>;
}

/// Stub backend that fails every call with `NotImplemented`. Lets the rest
/// of the workspace (orchestrator, controllers, tests) be developed against
/// the trait before a real CAST implementation is selected.
pub struct StubCastDriver;

impl CastDriver for StubCastDriver {
    async fn discover(&self) -> Result<Vec<DiscoveredCastDevice>, CastError> {
        Err(CastError::NotImplemented)
    }

    async fn play(
        &self,
        _target: &CastTarget,
        _media: &CastMedia,
    ) -> Result<CastSessionId, CastError> {
        Err(CastError::NotImplemented)
    }

    async fn control(&self, _session: &CastSessionId, _cmd: &ControlCmd) -> Result<(), CastError> {
        Err(CastError::NotImplemented)
    }

    async fn status_snapshot(&self, _session: &CastSessionId) -> Result<CastStatus, CastError> {
        Err(CastError::NotImplemented)
    }

    fn status_stream(&self, _session: &CastSessionId) -> BoxStream<'static, CastStatus> {
        Box::pin(futures::stream::empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn stub_returns_not_implemented() {
        let driver = StubCastDriver;
        match driver.discover().await {
            Err(CastError::NotImplemented) => {}
            other => panic!("expected NotImplemented, got {other:?}"),
        }
    }

    #[test]
    fn cast_session_id_is_unique() {
        let a = CastSessionId::new();
        let b = CastSessionId::new();
        assert_ne!(a, b);
    }

    #[test]
    fn control_cmd_round_trips_json() {
        let cmd = ControlCmd::Seek {
            position_secs: 42.5,
        };
        let json = serde_json::to_string(&cmd).expect("serialize");
        let back: ControlCmd = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(cmd, back);
    }
}
