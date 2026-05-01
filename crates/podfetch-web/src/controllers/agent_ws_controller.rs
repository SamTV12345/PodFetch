use crate::app_state::AppState;
use crate::events::CastEndedReason;
use crate::server::ChatServerHandle;
use crate::services::agent::registry::AgentSessionHandle;
use axum::Router;
use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::response::Response;
use axum::routing::get;
use chrono::Utc;
use futures::{SinkExt, StreamExt};
use podfetch_agent_protocol::{
    AgentMsg, ErrorCode, PROTOCOL_VERSION, ServerMsg,
};
use podfetch_domain::user::User;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn};

const SEND_BUFFER: usize = 32;

/// Mounts `GET /agent/ws` outside the `/api/v1` namespace and outside the
/// browser auth middleware — the handler does its own bearer-key auth
/// against the existing `users.api_key` column.
pub fn get_agent_ws_router() -> Router<AppState> {
    Router::new().route("/agent/ws", get(agent_ws_upgrade))
}

async fn agent_ws_upgrade(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum_extra::extract::OptionalQuery(query): axum_extra::extract::OptionalQuery<ApiKeyQuery>,
    ws: WebSocketUpgrade,
) -> Result<Response, StatusCode> {
    let api_key = extract_api_key(&headers, query.as_ref())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let user = state
        .user_auth_service
        .find_by_api_key(&api_key)
        .map_err(|e| {
            warn!("agent ws: api_key lookup failed: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    Ok(ws.on_upgrade(move |socket| handle_agent_socket(state, user, socket)))
}

#[derive(serde::Deserialize)]
struct ApiKeyQuery {
    api_key: Option<String>,
}

fn extract_api_key(headers: &HeaderMap, query: Option<&ApiKeyQuery>) -> Option<String> {
    if let Some(value) = headers.get(axum::http::header::AUTHORIZATION) {
        if let Ok(s) = value.to_str() {
            if let Some(rest) = s.strip_prefix("Bearer ") {
                return Some(rest.trim().to_string());
            }
        }
    }
    query.and_then(|q| q.api_key.clone())
}

async fn handle_agent_socket(state: AppState, user: User, socket: WebSocket) {
    let (mut ws_sink, mut ws_stream) = socket.split();

    if send_msg(
        &mut ws_sink,
        &ServerMsg::Hello {
            protocol_version: PROTOCOL_VERSION,
            server_version: env!("CARGO_PKG_VERSION").to_string(),
        },
    )
    .await
    .is_err()
    {
        return;
    }

    // Wait for HelloAck, with no other traffic in between.
    let agent_info = match ws_stream.next().await {
        Some(Ok(Message::Text(text))) => match serde_json::from_str::<AgentMsg>(&text) {
            Ok(AgentMsg::HelloAck {
                protocol_version,
                agent_id,
                agent_version,
                ..
            }) => {
                if protocol_version != PROTOCOL_VERSION {
                    let _ = send_msg(
                        &mut ws_sink,
                        &ServerMsg::Goodbye,
                    )
                    .await;
                    warn!(
                        "agent ws: protocol mismatch (server={PROTOCOL_VERSION}, agent={protocol_version})"
                    );
                    return;
                }
                (agent_id, agent_version)
            }
            other => {
                warn!("agent ws: expected HelloAck, got {other:?}");
                return;
            }
        },
        other => {
            warn!("agent ws: handshake aborted: {other:?}");
            return;
        }
    };

    let (agent_id, agent_version) = agent_info;

    let (tx, mut rx) = mpsc::channel::<ServerMsg>(SEND_BUFFER);
    let handle = AgentSessionHandle::new(agent_id.clone(), user.id, agent_version, tx);
    let displaced = state.agent_registry.register(handle);
    if displaced.is_some() {
        info!(agent_id = %agent_id, "displaced previous agent connection");
    }
    info!(agent_id = %agent_id, user_id = user.id, "agent connected");

    // Outbound pump: forwards anything pushed into the channel out over
    // the websocket.
    let outbound_agent_id = agent_id.clone();
    let outbound = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if send_msg(&mut ws_sink, &msg).await.is_err() {
                break;
            }
        }
        let _ = ws_sink.close().await;
        outbound_agent_id // returned for logging
    });

    // Inbound pump: process messages from the agent.
    while let Some(frame) = ws_stream.next().await {
        let text = match frame {
            Ok(Message::Text(t)) => t,
            Ok(Message::Binary(_)) => {
                warn!(agent_id = %agent_id, "agent ws: binary frame ignored");
                continue;
            }
            Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => continue,
            Ok(Message::Close(_)) => break,
            Err(err) => {
                warn!(agent_id = %agent_id, "agent ws: stream error: {err}");
                break;
            }
        };

        match serde_json::from_str::<AgentMsg>(&text) {
            Ok(msg) => handle_agent_message(&state, &agent_id, user.id, msg).await,
            Err(err) => {
                warn!(agent_id = %agent_id, "agent ws: bad json: {err}");
                let _ = state.agent_registry.send_to(
                    &agent_id,
                    ServerMsg::Goodbye,
                );
            }
        }
    }

    state.agent_registry.unregister(&agent_id);
    let _ = outbound.await;
    info!(agent_id = %agent_id, "agent disconnected");
}

async fn handle_agent_message(
    state: &AppState,
    agent_id: &str,
    user_id: i32,
    msg: AgentMsg,
) {
    match msg {
        AgentMsg::HelloAck { .. } => {
            warn!(agent_id, "unexpected duplicate HelloAck");
        }
        AgentMsg::DeviceList {
            request_id,
            ref devices,
        } => {
            for device in devices {
                let ip = device.ip.map(|ip| ip.to_string());
                if let Err(err) = state.device_service.upsert_chromecast_from_agent(
                    device.uuid.as_ref(),
                    agent_id,
                    user_id,
                    &device.friendly_name,
                    ip.as_deref(),
                    Utc::now().naive_utc(),
                ) {
                    warn!(
                        agent_id,
                        "failed to upsert device {}: {err}",
                        device.uuid.as_ref()
                    );
                }
            }
            // Solicited DeviceList → wake the dispatcher waiting on the
            // matching DiscoverRequest.
            if let Some(rid) = request_id.clone() {
                state.agent_dispatcher.complete_pending(
                    &rid,
                    AgentMsg::DeviceList {
                        request_id: Some(rid.clone()),
                        devices: devices.clone(),
                    },
                );
            }
        }
        AgentMsg::SessionStarted { ref request_id, .. } => {
            // Always correlated — forward to whoever issued the Play.
            let rid = request_id.clone();
            state.agent_dispatcher.complete_pending(&rid, msg);
        }
        AgentMsg::Status { status } => {
            // Update the cached snapshot and broadcast onward to the UI.
            if state.cast_orchestrator.record_status(status.clone()).is_some() {
                ChatServerHandle::broadcast_cast_status(status);
            }
        }
        AgentMsg::SessionEnded { session_id, reason } => {
            if state
                .cast_orchestrator
                .drop_session(&session_id)
                .is_some()
            {
                ChatServerHandle::broadcast_cast_ended(
                    session_id,
                    map_session_end_reason(reason),
                );
            }
        }
        AgentMsg::Pong => {}
        AgentMsg::Error {
            request_id,
            code,
            ref message,
        } => {
            if let Some(rid) = request_id.clone() {
                let rid_for_lookup = rid.clone();
                state.agent_dispatcher.complete_pending(
                    &rid_for_lookup,
                    AgentMsg::Error {
                        request_id: Some(rid),
                        code,
                        message: message.clone(),
                    },
                );
            } else {
                warn!(agent_id, "agent reported uncorrelated error {code:?}: {message}");
            }
        }
    }
    let _ = ErrorCode::InvalidRequest; // keep variant exercised
}

fn map_session_end_reason(
    reason: podfetch_agent_protocol::SessionEndReason,
) -> CastEndedReason {
    use podfetch_agent_protocol::SessionEndReason;
    match reason {
        SessionEndReason::Stopped => CastEndedReason::Stopped,
        SessionEndReason::Finished => CastEndedReason::Finished,
        SessionEndReason::DeviceGone => CastEndedReason::DeviceGone,
        SessionEndReason::Error => CastEndedReason::Error,
    }
}

async fn send_msg<S>(sink: &mut S, msg: &ServerMsg) -> Result<(), ()>
where
    S: SinkExt<Message, Error = axum::Error> + Unpin,
{
    let json = match serde_json::to_string(msg) {
        Ok(s) => s,
        Err(e) => {
            warn!("agent ws: failed to serialize {e}");
            return Err(());
        }
    };
    if let Err(e) = sink.send(Message::Text(json.into())).await {
        warn!("agent ws: send failed: {e}");
        return Err(());
    }
    Ok(())
}

// `Arc` import kept around for future extension where the registry is
// passed explicitly into background workers.
#[allow(dead_code)]
fn _arc_marker(r: Arc<()>) -> Arc<()> {
    r
}
