//! Local Chromecast driver wrapping the synchronous [`rust_cast`] crate.
//!
//! Each active cast session owns a dedicated OS thread that holds the
//! [`CastDevice`] for the duration of playback. The thread runs a poll
//! loop that interleaves command execution (Pause/Resume/Stop/Seek/
//! SetVolume) with `media.get_status` so the agent always knows the
//! current player state and position.
//!
//! Status updates flow out via a [`tokio::sync::mpsc`] channel
//! (`AgentEvent`) — the agent client drains that channel and forwards
//! `AgentMsg::Status` / `AgentMsg::SessionEnded` to the server.

use chrono::Utc;
use podfetch_agent_protocol::SessionEndReason;
use podfetch_cast::{
    CastMedia, CastSessionId, CastState, CastStatus, CastTarget, ControlCmd,
};
use rust_cast::CastDevice;
use rust_cast::channels::media::{IdleReason, Media, PlayerState, StreamType};
use rust_cast::channels::receiver::CastDeviceApp;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::Once;
#[cfg(test)]
use std::sync::Arc;
use std::sync::mpsc as std_mpsc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::mpsc as tokio_mpsc;
use tracing::{info, warn};

const RECEIVER_DESTINATION: &str = "receiver-0";
const STATUS_POLL_INTERVAL: Duration = Duration::from_millis(1500);

/// rustls 0.23 requires a crypto provider to be installed before any TLS
/// op. Workspaces that pull in both `aws-lc-rs` and `ring` (we do — via
/// reqwest + rust_cast) make auto-selection ambiguous, so we install one
/// explicitly the first time the driver is used. Idempotent via Once.
static INSTALL_CRYPTO: Once = Once::new();

fn ensure_crypto_provider() {
    INSTALL_CRYPTO.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

/// Events emitted by per-session workers and consumed by the agent's
/// websocket client. The client lifts each variant onto an [`AgentMsg`]
/// and pushes it to the server.
#[derive(Debug, Clone)]
pub enum AgentEvent {
    Status(CastStatus),
    SessionEnded {
        session_id: CastSessionId,
        reason: SessionEndReason,
    },
}

#[derive(Debug)]
enum WorkerCommand {
    Pause,
    Resume,
    Stop,
    Seek(f64),
    SetVolume(f32),
}

#[derive(Debug)]
struct ActiveSession {
    cmd_tx: std_mpsc::Sender<WorkerCommand>,
}

#[derive(Debug, Error)]
pub enum CastDriveError {
    #[error("device {host}:{port} unreachable: {reason}")]
    Connect {
        host: String,
        port: u16,
        reason: String,
    },
    #[error("CAST receiver rejected request: {0}")]
    Receiver(String),
    #[error("session {0:?} not active")]
    SessionGone(CastSessionId),
    #[error("internal: {0}")]
    Internal(String),
}

pub struct LocalCastDriver {
    sessions: Mutex<HashMap<CastSessionId, ActiveSession>>,
    event_tx: tokio_mpsc::Sender<AgentEvent>,
}

impl LocalCastDriver {
    pub fn new(event_tx: tokio_mpsc::Sender<AgentEvent>) -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            event_tx,
        }
    }

    /// Connect to the receiver, launch the Default Media Receiver app and
    /// load `media`. On success spawns a dedicated OS thread that keeps
    /// the connection alive for the duration of the session and starts
    /// streaming status events.
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
        let session_id = CastSessionId::new();
        let session_id_for_worker = session_id.clone();
        let event_tx = self.event_tx.clone();

        ensure_crypto_provider();

        let cmd_tx = tokio::task::spawn_blocking(move || -> Result<std_mpsc::Sender<WorkerCommand>, CastDriveError> {
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
                .ok_or_else(|| CastDriveError::Receiver("CAST status returned no entries".into()))?;

            let (cmd_tx, cmd_rx) = std_mpsc::channel::<WorkerCommand>();
            let transport_id = app.transport_id.clone();

            std::thread::Builder::new()
                .name(format!("podfetch-cast-{}", session_id_for_worker.0))
                .spawn(move || {
                    run_session_worker(
                        cast,
                        transport_id,
                        media_session_id,
                        session_id_for_worker,
                        cmd_rx,
                        event_tx,
                    );
                })
                .map_err(|e| CastDriveError::Internal(format!("spawn worker: {e}")))?;

            Ok(cmd_tx)
        })
        .await
        .map_err(|e| CastDriveError::Internal(format!("blocking task panic: {e}")))??;

        info!(session = %session_id.0, "CAST play succeeded; worker started");
        self.sessions
            .lock()
            .expect("driver session lock poisoned")
            .insert(session_id.clone(), ActiveSession { cmd_tx });
        Ok(session_id)
    }

    /// Forward a control command to the session's worker thread. Returns
    /// `SessionGone` if the worker has already exited or the session id
    /// is unknown.
    pub async fn control(
        &self,
        session_id: &CastSessionId,
        cmd: &ControlCmd,
    ) -> Result<(), CastDriveError> {
        let worker_cmd = match cmd {
            ControlCmd::Pause => WorkerCommand::Pause,
            ControlCmd::Resume => WorkerCommand::Resume,
            ControlCmd::Stop => WorkerCommand::Stop,
            ControlCmd::Seek { position_secs } => WorkerCommand::Seek(*position_secs),
            ControlCmd::SetVolume { volume } => WorkerCommand::SetVolume(*volume),
        };
        let mut sessions = self
            .sessions
            .lock()
            .expect("driver session lock poisoned");
        let entry = sessions
            .get(session_id)
            .ok_or_else(|| CastDriveError::SessionGone(session_id.clone()))?;
        let stop = matches!(worker_cmd, WorkerCommand::Stop);
        entry
            .cmd_tx
            .send(worker_cmd)
            .map_err(|_| CastDriveError::SessionGone(session_id.clone()))?;
        if stop {
            // Optimistically drop the entry; the worker will also emit
            // SessionEnded, which will be a noop on the orchestrator side.
            sessions.remove(session_id);
        }
        Ok(())
    }

    #[cfg(test)]
    pub fn knows_session(&self, session_id: &CastSessionId) -> bool {
        self.sessions
            .lock()
            .expect("driver session lock poisoned")
            .contains_key(session_id)
    }
}

fn run_session_worker(
    cast: CastDevice<'static>,
    transport_id: String,
    media_session_id: i32,
    session_id: CastSessionId,
    cmd_rx: std_mpsc::Receiver<WorkerCommand>,
    event_tx: tokio_mpsc::Sender<AgentEvent>,
) {
    let mut last_volume: f32 = 1.0;
    loop {
        // Drain any pending commands without blocking. We process at most
        // one per tick so status polling stays regular.
        match cmd_rx.try_recv() {
            Ok(cmd) => {
                if !apply_command(&cast, &transport_id, media_session_id, cmd, &mut last_volume) {
                    let _ = event_tx.blocking_send(AgentEvent::SessionEnded {
                        session_id: session_id.clone(),
                        reason: SessionEndReason::Stopped,
                    });
                    return;
                }
            }
            Err(std_mpsc::TryRecvError::Empty) => {}
            Err(std_mpsc::TryRecvError::Disconnected) => {
                // Driver dropped — nothing left to do.
                return;
            }
        }

        match cast
            .media
            .get_status(transport_id.as_str(), Some(media_session_id))
        {
            Ok(status) => {
                if let Some(entry) = status.entries.first() {
                    let state = map_player_state(entry.player_state);
                    let position = entry.current_time.unwrap_or(0.0) as f64;
                    let snapshot = CastStatus {
                        session_id: session_id.clone(),
                        state,
                        position_secs: position,
                        volume: last_volume,
                        at: Utc::now(),
                    };
                    if event_tx.blocking_send(AgentEvent::Status(snapshot)).is_err() {
                        return;
                    }
                    if state == CastState::Idle && entry.idle_reason.is_some() {
                        let reason = match entry.idle_reason {
                            Some(IdleReason::Finished) => SessionEndReason::Finished,
                            Some(IdleReason::Cancelled) => SessionEndReason::Stopped,
                            Some(IdleReason::Error) => SessionEndReason::Error,
                            _ => SessionEndReason::DeviceGone,
                        };
                        let _ = event_tx.blocking_send(AgentEvent::SessionEnded {
                            session_id,
                            reason,
                        });
                        return;
                    }
                }
            }
            Err(err) => {
                warn!(session = %session_id.0, "get_status failed: {err}");
                let _ = event_tx.blocking_send(AgentEvent::SessionEnded {
                    session_id,
                    reason: SessionEndReason::Error,
                });
                return;
            }
        }

        std::thread::sleep(STATUS_POLL_INTERVAL);
    }
}

/// Returns false iff the worker should exit after this command (Stop).
fn apply_command(
    cast: &CastDevice<'static>,
    transport_id: &str,
    media_session_id: i32,
    cmd: WorkerCommand,
    last_volume: &mut f32,
) -> bool {
    match cmd {
        WorkerCommand::Pause => {
            if let Err(e) = cast.media.pause(transport_id, media_session_id) {
                warn!("media.pause: {e}");
            }
            true
        }
        WorkerCommand::Resume => {
            if let Err(e) = cast.media.play(transport_id, media_session_id) {
                warn!("media.play: {e}");
            }
            true
        }
        WorkerCommand::Stop => {
            if let Err(e) = cast.media.stop(transport_id, media_session_id) {
                warn!("media.stop: {e}");
            }
            false
        }
        WorkerCommand::Seek(secs) => {
            if let Err(e) =
                cast.media
                    .seek(transport_id, media_session_id, Some(secs as f32), None)
            {
                warn!("media.seek: {e}");
            }
            true
        }
        WorkerCommand::SetVolume(level) => {
            if let Err(e) = cast.receiver.set_volume(level) {
                warn!("receiver.set_volume: {e}");
            } else {
                *last_volume = level;
            }
            true
        }
    }
}

fn map_player_state(state: PlayerState) -> CastState {
    match state {
        PlayerState::Idle => CastState::Idle,
        PlayerState::Playing => CastState::Playing,
        PlayerState::Buffering => CastState::Buffering,
        PlayerState::Paused => CastState::Paused,
    }
}

/// Test-only helper exposed for the rest of the crate.
#[cfg(test)]
pub fn driver_with_dummy_event_sink() -> (Arc<LocalCastDriver>, tokio_mpsc::Receiver<AgentEvent>) {
    let (tx, rx) = tokio_mpsc::channel(16);
    (Arc::new(LocalCastDriver::new(tx)), rx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use podfetch_cast::CastDeviceUuid;
    use std::net::{IpAddr, Ipv4Addr};

    fn unreachable_target() -> CastTarget {
        CastTarget {
            uuid: CastDeviceUuid("test".into()),
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
        let (driver, _rx) = driver_with_dummy_event_sink();
        let result = driver.play(&unreachable_target(), &media()).await;
        match result {
            Err(CastDriveError::Connect { .. }) => {}
            other => panic!("expected Connect error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn control_on_unknown_session_is_session_gone() {
        let (driver, _rx) = driver_with_dummy_event_sink();
        let result = driver
            .control(&CastSessionId("ghost".into()), &ControlCmd::Pause)
            .await;
        match result {
            Err(CastDriveError::SessionGone(_)) => {}
            other => panic!("expected SessionGone, got {other:?}"),
        }
    }

    #[test]
    fn unknown_session_is_not_tracked() {
        let (driver, _rx) = driver_with_dummy_event_sink();
        assert!(!driver.knows_session(&CastSessionId("nope".into())));
    }

    #[test]
    fn map_player_state_covers_all_variants() {
        assert_eq!(map_player_state(PlayerState::Idle), CastState::Idle);
        assert_eq!(map_player_state(PlayerState::Playing), CastState::Playing);
        assert_eq!(map_player_state(PlayerState::Buffering), CastState::Buffering);
        assert_eq!(map_player_state(PlayerState::Paused), CastState::Paused);
    }

    /// Used to make sure `CastDeviceUuid` is in scope and the import line
    /// stays alive — eliminates a dead-import warning when the rest of
    /// the file doesn't reference it.
    #[allow(dead_code)]
    fn _uuid_marker() -> CastDeviceUuid {
        CastDeviceUuid("x".into())
    }
}
