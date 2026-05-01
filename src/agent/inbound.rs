//! Handles inbound `ServerMsg` traffic on the agent side. Pure logic —
//! takes a message and produces zero or more outbound replies, so it can
//! be unit-tested without a real websocket.

use podfetch_agent_protocol::{
    AgentMsg, ErrorCode, ServerMsg,
};
use podfetch_cast::DiscoveredCastDevice;

/// Decision the dispatch loop should act on after handling a server msg.
#[derive(Debug, Clone, PartialEq)]
pub enum InboundOutcome {
    /// Send these messages back over the websocket, in order.
    Reply(Vec<AgentMsg>),
    /// Server is going away — gracefully shut down.
    Goodbye,
    /// Server sent something we don't understand — log and continue.
    Ignore,
}

/// Routes one inbound message. The current device list is passed in so
/// `DiscoverRequest` can be answered synchronously; future Phase 5c
/// versions will wire a real CAST driver to handle Play/Control.
pub fn dispatch(msg: ServerMsg, devices: &[DiscoveredCastDevice]) -> InboundOutcome {
    match msg {
        ServerMsg::Hello { .. } => {
            // The agent never expects Hello after the initial handshake;
            // duplicates are a server bug — ignore.
            InboundOutcome::Ignore
        }
        ServerMsg::Ping => InboundOutcome::Reply(vec![AgentMsg::Pong]),
        ServerMsg::Goodbye => InboundOutcome::Goodbye,
        ServerMsg::DiscoverRequest { request_id } => {
            InboundOutcome::Reply(vec![AgentMsg::DeviceList {
                request_id: Some(request_id),
                devices: devices.to_vec(),
            }])
        }
        ServerMsg::Play { request_id, .. } | ServerMsg::Control { request_id, .. } => {
            // Real CAST handling will land in Phase 5c. Until then, fail
            // fast so the orchestrator surfaces a clean error to the UI.
            InboundOutcome::Reply(vec![AgentMsg::Error {
                request_id: Some(request_id),
                code: ErrorCode::NotImplemented,
                message: "agent does not yet implement the CAST protocol".into(),
            }])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use podfetch_cast::CastDeviceUuid;

    fn dev() -> DiscoveredCastDevice {
        DiscoveredCastDevice {
            uuid: CastDeviceUuid("uuid-1".into()),
            friendly_name: "Living Room".into(),
            model: None,
            ip: None,
            port: 8009,
        }
    }

    #[test]
    fn ping_produces_pong() {
        let out = dispatch(ServerMsg::Ping, &[]);
        assert_eq!(out, InboundOutcome::Reply(vec![AgentMsg::Pong]));
    }

    #[test]
    fn goodbye_signals_shutdown() {
        assert_eq!(dispatch(ServerMsg::Goodbye, &[]), InboundOutcome::Goodbye);
    }

    #[test]
    fn discover_request_replies_with_current_devices() {
        let devices = vec![dev()];
        let out = dispatch(
            ServerMsg::DiscoverRequest {
                request_id: "req-1".into(),
            },
            &devices,
        );
        match out {
            InboundOutcome::Reply(msgs) => {
                assert_eq!(msgs.len(), 1);
                match &msgs[0] {
                    AgentMsg::DeviceList { request_id, devices: returned } => {
                        assert_eq!(request_id.as_deref(), Some("req-1"));
                        assert_eq!(returned.len(), 1);
                        assert_eq!(returned[0].uuid.as_ref(), "uuid-1");
                    }
                    other => panic!("expected DeviceList, got {other:?}"),
                }
            }
            other => panic!("expected Reply, got {other:?}"),
        }
    }

    #[test]
    fn play_returns_not_implemented_error_with_correlation() {
        let out = dispatch(
            ServerMsg::Play {
                request_id: "req-2".into(),
                chromecast_uuid: "uuid-1".into(),
                media: podfetch_agent_protocol::PlayMedia {
                    url: "x".into(),
                    mime: "audio/mpeg".into(),
                    title: "t".into(),
                    artwork_url: None,
                    duration_secs: None,
                    episode_id: None,
                },
            },
            &[],
        );
        match out {
            InboundOutcome::Reply(msgs) => match msgs.into_iter().next() {
                Some(AgentMsg::Error {
                    request_id,
                    code,
                    ..
                }) => {
                    assert_eq!(request_id.as_deref(), Some("req-2"));
                    assert_eq!(code, ErrorCode::NotImplemented);
                }
                other => panic!("expected Error reply, got {other:?}"),
            },
            other => panic!("expected Reply, got {other:?}"),
        }
    }
}
