//! Drains [`MopidyEvent`]s from the driver and feeds them into the cast
//! orchestrator — the in-process analogue of how `agent_ws_controller`
//! handles `AgentMsg::Status` / `SessionEnded` from the LAN agent.

use crate::app_state::AppState;
use crate::server::ChatServerHandle;
use crate::services::cast::service::ActiveSession;
use crate::services::mopidy::driver::MopidyEvent;
use crate::usecases::watchtime::WatchtimeUseCase;
use tokio::sync::mpsc;
use tracing::warn;

/// Spawn the consumer loop. Call once at startup when the integration is on.
pub fn spawn_status_consumer(state: AppState, mut rx: mpsc::Receiver<MopidyEvent>) {
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                MopidyEvent::Status(status) => {
                    if let Some(session) = state.cast_orchestrator.record_status(status.clone()) {
                        ChatServerHandle::broadcast_cast_status(status.clone());
                        persist_watchtime(&session, status.position_secs);
                    }
                }
                MopidyEvent::SessionEnded { session_id, reason } => {
                    if let Some(session) = state.cast_orchestrator.drop_session(&session_id) {
                        persist_watchtime(&session, session.last_status.position_secs);
                        ChatServerHandle::broadcast_cast_ended(session_id, reason);
                    }
                }
            }
        }
    });
}

fn persist_watchtime(session: &ActiveSession, position_secs: f64) {
    let Some(podcast_episode_id) = session.episode_string_id.clone() else {
        return;
    };
    let username = session.username.clone();
    let position = position_secs.max(0.0).min(f64::from(i32::MAX)) as i32;
    tokio::task::spawn_blocking(move || {
        if let Err(err) = WatchtimeUseCase::log_watchtime(&podcast_episode_id, position, username) {
            warn!("mopidy watchtime persist failed: {err}");
        }
    });
}
