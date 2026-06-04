//! In-process Mopidy playback driver. Sibling of `AgentDispatcher` — it is
//! deliberately NOT a `CastDriver` impl, because `CastTarget` is Chromecast
//! shaped (raw IpAddr + fixed port) and cannot represent a URL-addressed
//! server. It reuses the shared cast value types and emits status over a
//! channel that the startup-wired consumer drains into the orchestrator.

use crate::events::CastEndedReason;
use crate::services::mopidy::rpc::{
    self, MopidyRpcClient, control_to_call, ms_to_secs, state_from_str, volume_from_mopidy,
};
use chrono::Utc;
use podfetch_cast::{CastMedia, CastSessionId, CastState, CastStatus, ControlCmd};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::warn;

const POLL_INTERVAL: Duration = Duration::from_millis(1500);

/// Where a Mopidy play/control command is routed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MopidyTarget {
    pub base_url: String,
}

/// Events emitted by per-session pumps, drained by the consumer.
#[derive(Debug, Clone)]
pub enum MopidyEvent {
    Status(CastStatus),
    SessionEnded {
        session_id: CastSessionId,
        reason: CastEndedReason,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum MopidyDriveError {
    #[error("mopidy rpc: {0}")]
    Rpc(#[from] rpc::MopidyRpcError),
    #[error("session {0:?} not active")]
    SessionGone(CastSessionId),
}

struct ActiveMopidySession {
    base_url: String,
    cancel: tokio::sync::watch::Sender<bool>,
}

pub struct MopidyDriver {
    event_tx: mpsc::Sender<MopidyEvent>,
    sessions: Mutex<HashMap<CastSessionId, ActiveMopidySession>>,
}

/// Decide whether a freshly polled state means the session ended.
/// `has_played` is true once we have observed a non-stopped state, so an
/// initial `Stopped` while buffering does not end the session prematurely.
pub fn end_reason_for_poll(has_played: bool, state: CastState) -> Option<CastEndedReason> {
    if has_played && state == CastState::Stopped {
        Some(CastEndedReason::Finished)
    } else {
        None
    }
}

impl MopidyDriver {
    pub fn new(event_tx: mpsc::Sender<MopidyEvent>) -> Self {
        Self {
            event_tx,
            sessions: Mutex::new(HashMap::new()),
        }
    }

    /// Connection test used by the management API. Returns the version string.
    pub async fn ping(base_url: &str) -> Result<String, MopidyDriveError> {
        let client = MopidyRpcClient::new(base_url);
        let v = client.call("core.get_version", json!({})).await?;
        Ok(v.as_str().unwrap_or_default().to_string())
    }

    pub async fn play(
        &self,
        target: &MopidyTarget,
        media: &CastMedia,
        resume_secs: Option<f64>,
    ) -> Result<CastSessionId, MopidyDriveError> {
        let client = MopidyRpcClient::new(&target.base_url);
        client.call("core.tracklist.clear", json!({})).await?;
        client
            .call("core.tracklist.add", json!({ "uris": [media.url] }))
            .await?;
        client.call("core.playback.play", json!({})).await?;
        if let Some(secs) = resume_secs.filter(|s| *s > 0.0) {
            let (method, params) = control_to_call(&ControlCmd::Seek { position_secs: secs });
            let _ = client.call(method, params).await;
        }

        let session_id = CastSessionId::new();
        let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(false);
        self.sessions
            .lock()
            .expect("mopidy session lock poisoned")
            .insert(
                session_id.clone(),
                ActiveMopidySession {
                    base_url: target.base_url.clone(),
                    cancel: cancel_tx,
                },
            );

        let event_tx = self.event_tx.clone();
        let base_url = target.base_url.clone();
        let pump_session = session_id.clone();
        tokio::spawn(async move {
            run_pump(base_url, pump_session, event_tx, cancel_rx).await;
        });

        Ok(session_id)
    }

    pub async fn control(
        &self,
        session_id: &CastSessionId,
        cmd: &ControlCmd,
    ) -> Result<(), MopidyDriveError> {
        let base_url = {
            let sessions = self.sessions.lock().expect("mopidy session lock poisoned");
            sessions
                .get(session_id)
                .map(|s| s.base_url.clone())
                .ok_or_else(|| MopidyDriveError::SessionGone(session_id.clone()))?
        };
        let client = MopidyRpcClient::new(&base_url);
        let (method, params) = control_to_call(cmd);
        client.call(method, params).await?;

        if matches!(cmd, ControlCmd::Stop) {
            self.finish_session(session_id, CastEndedReason::Stopped);
        }
        Ok(())
    }

    fn finish_session(&self, session_id: &CastSessionId, reason: CastEndedReason) {
        if let Some(session) = self
            .sessions
            .lock()
            .expect("mopidy session lock poisoned")
            .remove(session_id)
        {
            let _ = session.cancel.send(true);
            let _ = self.event_tx.try_send(MopidyEvent::SessionEnded {
                session_id: session_id.clone(),
                reason,
            });
        }
    }

    #[cfg(test)]
    pub fn knows_session(&self, session_id: &CastSessionId) -> bool {
        self.sessions
            .lock()
            .expect("mopidy session lock poisoned")
            .contains_key(session_id)
    }
}

async fn run_pump(
    base_url: String,
    session_id: CastSessionId,
    event_tx: mpsc::Sender<MopidyEvent>,
    mut cancel_rx: tokio::sync::watch::Receiver<bool>,
) {
    let client = MopidyRpcClient::new(&base_url);
    let mut has_played = false;
    loop {
        if *cancel_rx.borrow() {
            return;
        }
        let snapshot = poll_once(&client).await;
        if let Some((state, position_secs, volume)) = snapshot {
            if state != CastState::Stopped {
                has_played = true;
            }
            let status = CastStatus {
                session_id: session_id.clone(),
                state,
                position_secs,
                volume,
                at: Utc::now(),
            };
            if event_tx.send(MopidyEvent::Status(status)).await.is_err() {
                return;
            }
            if let Some(reason) = end_reason_for_poll(has_played, state) {
                let _ = event_tx
                    .send(MopidyEvent::SessionEnded { session_id, reason })
                    .await;
                return;
            }
        }
        tokio::select! {
            _ = tokio::time::sleep(POLL_INTERVAL) => {}
            _ = cancel_rx.changed() => return,
        }
    }
}

/// One poll cycle → `(state, position_secs, volume)`. `None` on transport error.
async fn poll_once(client: &MopidyRpcClient) -> Option<(CastState, f64, f32)> {
    let state = match client.call("core.playback.get_state", json!({})).await {
        Ok(Value::String(s)) => state_from_str(&s),
        Ok(_) => CastState::Stopped,
        Err(e) => {
            warn!("mopidy get_state failed: {e}");
            return None;
        }
    };
    let position = client
        .call("core.playback.get_time_position", json!({}))
        .await
        .ok()
        .and_then(|v| v.as_i64())
        .map(ms_to_secs)
        .unwrap_or(0.0);
    let volume = client
        .call("core.mixer.get_volume", json!({}))
        .await
        .ok()
        .and_then(|v| v.as_i64())
        .map(volume_from_mopidy)
        .unwrap_or(1.0);
    Some((state, position, volume))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn end_reason_ignores_initial_stopped_then_finishes() {
        assert_eq!(end_reason_for_poll(false, CastState::Stopped), None);
        assert_eq!(end_reason_for_poll(true, CastState::Playing), None);
        assert_eq!(
            end_reason_for_poll(true, CastState::Stopped),
            Some(CastEndedReason::Finished)
        );
    }

    #[tokio::test]
    async fn control_on_unknown_session_is_session_gone() {
        let (tx, _rx) = mpsc::channel(4);
        let driver = MopidyDriver::new(tx);
        let err = driver
            .control(&CastSessionId("ghost".into()), &ControlCmd::Pause)
            .await;
        assert!(matches!(err, Err(MopidyDriveError::SessionGone(_))));
    }
}
