use crate::services::agent::registry::AgentRegistry;
use podfetch_agent_protocol::{AgentMsg, ErrorCode, PlayMedia, RequestId, ServerMsg};
use podfetch_cast::{
    CastDeviceUuid, CastError, CastMedia, CastSessionId, CastTarget, ControlCmd,
    DiscoveredCastDevice,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::oneshot;
use tracing::warn;

/// How long the orchestrator waits for a correlated reply from the agent
/// before giving up and surfacing a transport error.
pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

/// Routes Chromecast operations onto a connected agent's websocket and
/// awaits the correlated reply. One instance per PodFetch process; lives
/// alongside the [`AgentRegistry`] in `AppState`.
pub struct AgentDispatcher {
    registry: Arc<AgentRegistry>,
    pending: Mutex<HashMap<RequestId, oneshot::Sender<AgentMsg>>>,
    request_timeout: Duration,
}

impl AgentDispatcher {
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self::with_timeout(registry, DEFAULT_REQUEST_TIMEOUT)
    }

    pub fn with_timeout(registry: Arc<AgentRegistry>, request_timeout: Duration) -> Self {
        Self {
            registry,
            pending: Mutex::new(HashMap::new()),
            request_timeout,
        }
    }

    /// Called by the WS handler when an agent message carries a request_id
    /// the dispatcher might be waiting on. If nothing is waiting the
    /// message is silently dropped — that's the right behaviour for late
    /// or duplicate replies after a timeout.
    pub fn complete_pending(&self, request_id: &str, msg: AgentMsg) {
        let sender = self
            .pending
            .lock()
            .expect("dispatcher pending lock poisoned")
            .remove(request_id);
        if let Some(sender) = sender {
            // The waiter may have already given up if we're racing a
            // timeout — that's not an error.
            let _ = sender.send(msg);
        } else {
            warn!(request_id, "agent reply for unknown request_id — ignored");
        }
    }

    /// Forward a Play to a specific agent and await its `SessionStarted`
    /// reply. Returns a transport error if the agent is not connected.
    pub async fn play(
        &self,
        agent_id: &str,
        target: &CastTarget,
        media: &CastMedia,
    ) -> Result<CastSessionId, CastError> {
        let request_id = uuid::Uuid::new_v4().to_string();
        let rx = self.register_pending(&request_id);

        let payload = ServerMsg::Play {
            request_id: request_id.clone(),
            chromecast_uuid: target.uuid.0.clone(),
            media: PlayMedia {
                url: media.url.clone(),
                mime: media.mime.clone(),
                title: media.title.clone(),
                artwork_url: media.artwork_url.clone(),
                duration_secs: media.duration_secs,
                episode_id: media.episode_id,
            },
        };
        if !self.registry.send_to(agent_id, payload) {
            self.discard_pending(&request_id);
            return Err(CastError::Transport(format!(
                "agent {agent_id} not connected"
            )));
        }

        match self.await_reply(rx, &request_id).await? {
            AgentMsg::SessionStarted { session_id, .. } => Ok(session_id),
            AgentMsg::Error { code, message, .. } => Err(map_agent_error(
                code,
                message,
                Some(target.uuid.clone()),
                None,
            )),
            other => Err(CastError::Transport(format!(
                "unexpected agent reply to Play: {other:?}"
            ))),
        }
    }

    pub async fn control(
        &self,
        agent_id: &str,
        session_id: &CastSessionId,
        cmd: &ControlCmd,
    ) -> Result<(), CastError> {
        let request_id = uuid::Uuid::new_v4().to_string();
        let rx = self.register_pending(&request_id);

        let payload = ServerMsg::Control {
            request_id: request_id.clone(),
            session_id: session_id.clone(),
            cmd: cmd.clone(),
        };
        if !self.registry.send_to(agent_id, payload) {
            self.discard_pending(&request_id);
            return Err(CastError::Transport(format!(
                "agent {agent_id} not connected"
            )));
        }

        match self.await_reply(rx, &request_id).await? {
            // Agents may either reply with a Status snapshot (treated as
            // success) or an Error.
            AgentMsg::Status { .. } | AgentMsg::SessionEnded { .. } => Ok(()),
            AgentMsg::Error { code, message, .. } => Err(map_agent_error(
                code,
                message,
                None,
                Some(session_id.clone()),
            )),
            // Some receivers don't bother to reply at all on success — we
            // accept that too, but here we got *something* unexpected.
            other => Err(CastError::Transport(format!(
                "unexpected agent reply to Control: {other:?}"
            ))),
        }
    }

    pub async fn discover(&self, agent_id: &str) -> Result<Vec<DiscoveredCastDevice>, CastError> {
        let request_id = uuid::Uuid::new_v4().to_string();
        let rx = self.register_pending(&request_id);

        let payload = ServerMsg::DiscoverRequest {
            request_id: request_id.clone(),
        };
        if !self.registry.send_to(agent_id, payload) {
            self.discard_pending(&request_id);
            return Err(CastError::Transport(format!(
                "agent {agent_id} not connected"
            )));
        }

        match self.await_reply(rx, &request_id).await? {
            AgentMsg::DeviceList { devices, .. } => Ok(devices),
            AgentMsg::Error { code, message, .. } => {
                Err(map_agent_error(code, message, None, None))
            }
            other => Err(CastError::Transport(format!(
                "unexpected agent reply to DiscoverRequest: {other:?}"
            ))),
        }
    }

    fn register_pending(&self, request_id: &str) -> oneshot::Receiver<AgentMsg> {
        let (tx, rx) = oneshot::channel();
        self.pending
            .lock()
            .expect("dispatcher pending lock poisoned")
            .insert(request_id.to_string(), tx);
        rx
    }

    fn discard_pending(&self, request_id: &str) {
        self.pending
            .lock()
            .expect("dispatcher pending lock poisoned")
            .remove(request_id);
    }

    async fn await_reply(
        &self,
        rx: oneshot::Receiver<AgentMsg>,
        request_id: &str,
    ) -> Result<AgentMsg, CastError> {
        match tokio::time::timeout(self.request_timeout, rx).await {
            Ok(Ok(msg)) => Ok(msg),
            Ok(Err(_)) => {
                // Sender dropped — the WS task closed without replying.
                self.discard_pending(request_id);
                Err(CastError::Transport(
                    "agent connection closed before reply".into(),
                ))
            }
            Err(_) => {
                self.discard_pending(request_id);
                Err(CastError::Transport("agent request timed out".into()))
            }
        }
    }
}

fn map_agent_error(
    code: ErrorCode,
    message: String,
    target: Option<CastDeviceUuid>,
    session: Option<CastSessionId>,
) -> CastError {
    match code {
        ErrorCode::DeviceNotFound => {
            CastError::DeviceNotFound(target.unwrap_or(CastDeviceUuid("?".into())))
        }
        ErrorCode::SessionNotFound => {
            CastError::SessionNotFound(session.unwrap_or(CastSessionId("?".into())))
        }
        ErrorCode::Receiver => CastError::Receiver(message),
        ErrorCode::Transport => CastError::Transport(message),
        ErrorCode::NotImplemented => CastError::NotImplemented,
        ErrorCode::InvalidRequest => CastError::Receiver(message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::agent::registry::AgentSessionHandle;
    use podfetch_agent_protocol::AgentCapabilities;
    use std::net::Ipv4Addr;
    use tokio::sync::mpsc;

    fn registry_with_agent(agent_id: &str) -> (Arc<AgentRegistry>, mpsc::Receiver<ServerMsg>) {
        let registry = Arc::new(AgentRegistry::new());
        let (tx, rx) = mpsc::channel(16);
        registry.register(AgentSessionHandle::new(
            agent_id.into(),
            1,
            "0.1.0".into(),
            tx,
        ));
        (registry, rx)
    }

    fn target(uuid: &str) -> CastTarget {
        CastTarget {
            uuid: CastDeviceUuid(uuid.into()),
            ip: std::net::IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)),
            port: 8009,
        }
    }

    fn media() -> CastMedia {
        CastMedia {
            url: "https://example.com/audio.mp3".into(),
            mime: "audio/mpeg".into(),
            title: "Ep 1".into(),
            artwork_url: None,
            duration_secs: Some(60.0),
            episode_id: Some(7),
        }
    }

    fn extract_request_id(msg: &ServerMsg) -> RequestId {
        match msg {
            ServerMsg::Play { request_id, .. }
            | ServerMsg::Control { request_id, .. }
            | ServerMsg::DiscoverRequest { request_id } => request_id.clone(),
            _ => panic!("expected correlated request, got {msg:?}"),
        }
    }

    #[tokio::test]
    async fn play_resolves_on_session_started() {
        let (registry, mut wire) = registry_with_agent("agent-1");
        let dispatcher = Arc::new(AgentDispatcher::new(registry));
        let target = target("uuid-A");

        let dispatcher_clone = dispatcher.clone();
        let task =
            tokio::spawn(async move { dispatcher_clone.play("agent-1", &target, &media()).await });

        // Read what the dispatcher sent over the channel.
        let sent = wire.recv().await.expect("server msg");
        let request_id = extract_request_id(&sent);

        // Reply with a SessionStarted.
        dispatcher.complete_pending(
            &request_id,
            AgentMsg::SessionStarted {
                request_id: request_id.clone(),
                session_id: CastSessionId("session-xyz".into()),
            },
        );

        let session_id = task.await.unwrap().unwrap();
        assert_eq!(session_id.0, "session-xyz");
    }

    #[tokio::test]
    async fn play_against_disconnected_agent_returns_transport_error() {
        let registry = Arc::new(AgentRegistry::new());
        let dispatcher = AgentDispatcher::new(registry);
        let result = dispatcher.play("nope", &target("uuid-A"), &media()).await;
        match result {
            Err(CastError::Transport(_)) => {}
            other => panic!("expected Transport, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn play_times_out_when_agent_never_replies() {
        let (registry, _wire) = registry_with_agent("agent-1");
        let dispatcher = AgentDispatcher::with_timeout(registry, Duration::from_millis(50));

        let result = dispatcher
            .play("agent-1", &target("uuid-A"), &media())
            .await;
        match result {
            Err(CastError::Transport(msg)) if msg.contains("timed out") => {}
            other => panic!("expected timeout transport error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn play_maps_agent_error_to_cast_error() {
        let (registry, mut wire) = registry_with_agent("agent-1");
        let dispatcher = Arc::new(AgentDispatcher::new(registry));
        let dispatcher_clone = dispatcher.clone();
        let task = tokio::spawn(async move {
            dispatcher_clone
                .play("agent-1", &target("uuid-A"), &media())
                .await
        });
        let request_id = extract_request_id(&wire.recv().await.unwrap());
        dispatcher.complete_pending(
            &request_id,
            AgentMsg::Error {
                request_id: Some(request_id.clone()),
                code: ErrorCode::DeviceNotFound,
                message: "no such device".into(),
            },
        );
        match task.await.unwrap() {
            Err(CastError::DeviceNotFound(_)) => {}
            other => panic!("expected DeviceNotFound, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn discover_returns_device_list() {
        let (registry, mut wire) = registry_with_agent("agent-1");
        let dispatcher = Arc::new(AgentDispatcher::new(registry));
        let dispatcher_clone = dispatcher.clone();
        let task = tokio::spawn(async move { dispatcher_clone.discover("agent-1").await });
        let request_id = extract_request_id(&wire.recv().await.unwrap());
        dispatcher.complete_pending(
            &request_id,
            AgentMsg::DeviceList {
                request_id: Some(request_id.clone()),
                devices: vec![DiscoveredCastDevice {
                    uuid: CastDeviceUuid("uuid-found".into()),
                    friendly_name: "Living Room".into(),
                    model: Some("Chromecast Audio".into()),
                    ip: None,
                    port: 8009,
                }],
            },
        );
        let devices = task.await.unwrap().unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].uuid.0, "uuid-found");
    }

    #[test]
    fn complete_pending_for_unknown_request_is_a_noop() {
        let registry = Arc::new(AgentRegistry::new());
        let dispatcher = AgentDispatcher::new(registry);
        // No panic, no observable change — just a warn log.
        dispatcher.complete_pending(
            "ghost",
            AgentMsg::Error {
                request_id: Some("ghost".into()),
                code: ErrorCode::Receiver,
                message: "stale".into(),
            },
        );
        let _ = AgentCapabilities {
            chromecast: true,
            local_proxy: true,
        }; // keep type referenced
    }
}
