use podfetch_agent_protocol::ServerMsg;
use std::collections::HashMap;
use std::sync::RwLock;
use tokio::sync::mpsc;
use tracing::warn;

/// Per-connection state held by the registry. The registry never owns the
/// websocket directly — the WS task does — but it does own the sender half
/// of an mpsc through which the WS task receives outgoing frames.
#[derive(Debug)]
pub struct AgentSessionHandle {
    pub agent_id: String,
    pub user_id: i32,
    pub agent_version: String,
    sender: mpsc::Sender<ServerMsg>,
}

impl AgentSessionHandle {
    pub fn new(
        agent_id: String,
        user_id: i32,
        agent_version: String,
        sender: mpsc::Sender<ServerMsg>,
    ) -> Self {
        Self {
            agent_id,
            user_id,
            agent_version,
            sender,
        }
    }

    /// Best-effort send. Returns `false` if the channel is full or the WS
    /// task has gone away (in which case the caller should treat the agent
    /// as disconnected and call [`AgentRegistry::unregister`]).
    pub fn try_send(&self, msg: ServerMsg) -> bool {
        match self.sender.try_send(msg) {
            Ok(()) => true,
            Err(mpsc::error::TrySendError::Full(_)) => {
                warn!(agent_id = %self.agent_id, "agent send buffer full — dropping message");
                false
            }
            Err(mpsc::error::TrySendError::Closed(_)) => false,
        }
    }
}

/// In-memory registry of currently-connected agents. Each running PodFetch
/// instance has exactly one `AgentRegistry`, attached to `AppState`.
#[derive(Debug, Default)]
pub struct AgentRegistry {
    inner: RwLock<HashMap<String, AgentSessionHandle>>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert (or replace) a connection for `agent_id`. Returns the
    /// previously-registered handle, if any — the caller should drop it to
    /// signal the older WS task to shut down.
    pub fn register(&self, handle: AgentSessionHandle) -> Option<AgentSessionHandle> {
        let mut guard = self.inner.write().expect("agent registry poisoned");
        guard.insert(handle.agent_id.clone(), handle)
    }

    pub fn unregister(&self, agent_id: &str) -> Option<AgentSessionHandle> {
        let mut guard = self.inner.write().expect("agent registry poisoned");
        guard.remove(agent_id)
    }

    /// Snapshot of currently-known agent ids (for diagnostics / admin UI).
    pub fn list_agent_ids(&self) -> Vec<String> {
        let guard = self.inner.read().expect("agent registry poisoned");
        guard.keys().cloned().collect()
    }

    /// True iff an agent with the given id is currently connected.
    pub fn is_connected(&self, agent_id: &str) -> bool {
        let guard = self.inner.read().expect("agent registry poisoned");
        guard.contains_key(agent_id)
    }

    /// Send `msg` to a specific agent. Returns false if the agent is not
    /// connected or the channel is closed/full.
    pub fn send_to(&self, agent_id: &str, msg: ServerMsg) -> bool {
        let guard = self.inner.read().expect("agent registry poisoned");
        match guard.get(agent_id) {
            Some(handle) => handle.try_send(msg),
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use podfetch_agent_protocol::ServerMsg;

    fn make_handle(agent_id: &str) -> (AgentSessionHandle, mpsc::Receiver<ServerMsg>) {
        let (tx, rx) = mpsc::channel(8);
        (
            AgentSessionHandle::new(agent_id.into(), 1, "0.1.0".into(), tx),
            rx,
        )
    }

    #[test]
    fn register_and_lookup() {
        let registry = AgentRegistry::new();
        let (handle, _rx) = make_handle("a1");
        assert!(registry.register(handle).is_none());
        assert!(registry.is_connected("a1"));
        assert_eq!(registry.list_agent_ids(), vec!["a1"]);
    }

    #[test]
    fn re_register_replaces_old_handle() {
        let registry = AgentRegistry::new();
        let (h1, _rx1) = make_handle("a1");
        registry.register(h1);
        let (h2, _rx2) = make_handle("a1");
        let displaced = registry.register(h2);
        assert!(displaced.is_some());
    }

    #[test]
    fn unregister_removes_from_registry() {
        let registry = AgentRegistry::new();
        let (h, _rx) = make_handle("a1");
        registry.register(h);
        assert!(registry.unregister("a1").is_some());
        assert!(!registry.is_connected("a1"));
        assert!(registry.unregister("a1").is_none());
    }

    #[tokio::test]
    async fn send_to_delivers_message_through_channel() {
        let registry = AgentRegistry::new();
        let (h, mut rx) = make_handle("a1");
        registry.register(h);

        assert!(registry.send_to("a1", ServerMsg::Ping));
        match rx.recv().await {
            Some(ServerMsg::Ping) => {}
            other => panic!("expected Ping, got {other:?}"),
        }
    }

    #[test]
    fn send_to_unknown_agent_returns_false() {
        let registry = AgentRegistry::new();
        assert!(!registry.send_to("nope", ServerMsg::Ping));
    }
}
