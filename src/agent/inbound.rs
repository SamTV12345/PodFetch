//! Routes inbound `ServerMsg` traffic on the agent side. Owns the
//! [`LocalCastDriver`] so Play/Control can actually drive the receiver.

use crate::agent::cast::{CastDriveError, LocalCastDriver};
use podfetch_agent_protocol::{
    AgentMsg, ErrorCode, PlayMedia, ServerMsg,
};
use podfetch_cast::{
    CastDeviceUuid, CastMedia, CastTarget, ControlCmd, DiscoveredCastDevice,
};
use std::sync::Arc;

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

/// Holds whatever the dispatcher needs to handle messages — currently just
/// the Cast driver. Cheap to clone (Arc inside).
#[derive(Clone)]
pub struct AgentService {
    cast: Arc<LocalCastDriver>,
}

impl AgentService {
    pub fn new(cast: Arc<LocalCastDriver>) -> Self {
        Self { cast }
    }

    /// Routes one inbound message. Discovered-device snapshot is passed in
    /// so `DiscoverRequest` answers with current data.
    pub async fn handle(
        &self,
        msg: ServerMsg,
        devices: &[DiscoveredCastDevice],
    ) -> InboundOutcome {
        match msg {
            ServerMsg::Hello { .. } => InboundOutcome::Ignore,
            ServerMsg::Ping => InboundOutcome::Reply(vec![AgentMsg::Pong]),
            ServerMsg::Goodbye => InboundOutcome::Goodbye,
            ServerMsg::DiscoverRequest { request_id } => {
                InboundOutcome::Reply(vec![AgentMsg::DeviceList {
                    request_id: Some(request_id),
                    devices: devices.to_vec(),
                }])
            }
            ServerMsg::Play {
                request_id,
                chromecast_uuid,
                media,
            } => self.handle_play(request_id, chromecast_uuid, media, devices).await,
            ServerMsg::Control { request_id, session_id, cmd } => {
                self.handle_control(request_id, session_id, cmd).await
            }
        }
    }

    async fn handle_play(
        &self,
        request_id: String,
        chromecast_uuid: String,
        media: PlayMedia,
        devices: &[DiscoveredCastDevice],
    ) -> InboundOutcome {
        let target = match resolve_target(&chromecast_uuid, devices) {
            Some(t) => t,
            None => {
                return InboundOutcome::Reply(vec![AgentMsg::Error {
                    request_id: Some(request_id),
                    code: ErrorCode::DeviceNotFound,
                    message: format!("device {chromecast_uuid} not in discovery snapshot"),
                }]);
            }
        };
        let cast_media = CastMedia {
            url: media.url,
            mime: media.mime,
            title: media.title,
            artwork_url: media.artwork_url,
            duration_secs: media.duration_secs,
            episode_id: media.episode_id,
        };

        match self.cast.play(&target, &cast_media).await {
            Ok(session_id) => InboundOutcome::Reply(vec![AgentMsg::SessionStarted {
                request_id,
                session_id,
            }]),
            Err(err) => InboundOutcome::Reply(vec![AgentMsg::Error {
                request_id: Some(request_id),
                code: cast_error_code(&err),
                message: err.to_string(),
            }]),
        }
    }

    async fn handle_control(
        &self,
        request_id: String,
        session_id: podfetch_cast::CastSessionId,
        cmd: ControlCmd,
    ) -> InboundOutcome {
        match self.cast.control(&session_id, &cmd).await {
            Ok(()) => InboundOutcome::Reply(vec![AgentMsg::SessionStarted {
                // No dedicated success ack in the protocol yet; reusing
                // SessionStarted as a "done" signal. Pause/Resume/Stop/Seek
                // currently always return NotImplemented anyway.
                request_id,
                session_id,
            }]),
            Err(err) => InboundOutcome::Reply(vec![AgentMsg::Error {
                request_id: Some(request_id),
                code: cast_error_code(&err),
                message: err.to_string(),
            }]),
        }
    }
}

fn resolve_target(
    chromecast_uuid: &str,
    devices: &[DiscoveredCastDevice],
) -> Option<CastTarget> {
    devices
        .iter()
        .find(|d| d.uuid.as_ref() == chromecast_uuid)
        .and_then(|d| {
            Some(CastTarget {
                uuid: CastDeviceUuid(d.uuid.0.clone()),
                ip: d.ip?,
                port: d.port,
            })
        })
}

fn cast_error_code(err: &CastDriveError) -> ErrorCode {
    match err {
        CastDriveError::Connect { .. } => ErrorCode::Transport,
        CastDriveError::Receiver(_) => ErrorCode::Receiver,
        CastDriveError::NotImplemented(_) => ErrorCode::NotImplemented,
        CastDriveError::Internal(_) => ErrorCode::Receiver,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    fn dev(uuid: &str) -> DiscoveredCastDevice {
        DiscoveredCastDevice {
            uuid: CastDeviceUuid(uuid.into()),
            friendly_name: "Living Room".into(),
            model: None,
            ip: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 50))),
            port: 8009,
        }
    }

    fn service() -> AgentService {
        AgentService::new(Arc::new(LocalCastDriver::new()))
    }

    fn play_msg(uuid: &str, request_id: &str) -> ServerMsg {
        ServerMsg::Play {
            request_id: request_id.into(),
            chromecast_uuid: uuid.into(),
            media: PlayMedia {
                url: "https://example.com/audio.mp3".into(),
                mime: "audio/mpeg".into(),
                title: "Ep".into(),
                artwork_url: None,
                duration_secs: Some(60.0),
                episode_id: Some(1),
            },
        }
    }

    #[tokio::test]
    async fn ping_produces_pong() {
        let svc = service();
        assert_eq!(
            svc.handle(ServerMsg::Ping, &[]).await,
            InboundOutcome::Reply(vec![AgentMsg::Pong])
        );
    }

    #[tokio::test]
    async fn goodbye_signals_shutdown() {
        let svc = service();
        assert_eq!(
            svc.handle(ServerMsg::Goodbye, &[]).await,
            InboundOutcome::Goodbye
        );
    }

    #[tokio::test]
    async fn discover_request_replies_with_current_devices() {
        let svc = service();
        let out = svc
            .handle(
                ServerMsg::DiscoverRequest {
                    request_id: "req-1".into(),
                },
                &[dev("uuid-1")],
            )
            .await;
        match out {
            InboundOutcome::Reply(msgs) => match &msgs[0] {
                AgentMsg::DeviceList { request_id, devices } => {
                    assert_eq!(request_id.as_deref(), Some("req-1"));
                    assert_eq!(devices.len(), 1);
                }
                other => panic!("unexpected: {other:?}"),
            },
            other => panic!("expected Reply, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn play_against_unknown_device_returns_device_not_found() {
        let svc = service();
        let out = svc
            .handle(play_msg("missing", "req-1"), &[dev("uuid-1")])
            .await;
        match out {
            InboundOutcome::Reply(msgs) => match &msgs[0] {
                AgentMsg::Error { code, request_id, .. } => {
                    assert_eq!(*code, ErrorCode::DeviceNotFound);
                    assert_eq!(request_id.as_deref(), Some("req-1"));
                }
                other => panic!("unexpected: {other:?}"),
            },
            other => panic!("expected Reply, got {other:?}"),
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn play_against_unreachable_device_returns_transport_error() {
        let svc = service();
        // Use a device with a reserved/unroutable IP so the CAST connect
        // attempt fails fast.
        let unreachable = DiscoveredCastDevice {
            uuid: CastDeviceUuid("uuid-1".into()),
            friendly_name: "Unreachable".into(),
            model: None,
            ip: Some(IpAddr::V4(Ipv4Addr::new(240, 0, 0, 1))),
            port: 8009,
        };
        let out = svc
            .handle(play_msg("uuid-1", "req-2"), &[unreachable])
            .await;
        match out {
            InboundOutcome::Reply(msgs) => match &msgs[0] {
                AgentMsg::Error { code, .. } => {
                    assert_eq!(*code, ErrorCode::Transport);
                }
                other => panic!("unexpected: {other:?}"),
            },
            other => panic!("expected Reply, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn control_returns_not_implemented_error() {
        let svc = service();
        let out = svc
            .handle(
                ServerMsg::Control {
                    request_id: "req-3".into(),
                    session_id: podfetch_cast::CastSessionId("sess".into()),
                    cmd: ControlCmd::Pause,
                },
                &[],
            )
            .await;
        match out {
            InboundOutcome::Reply(msgs) => match &msgs[0] {
                AgentMsg::Error { code, .. } => {
                    assert_eq!(*code, ErrorCode::NotImplemented);
                }
                other => panic!("unexpected: {other:?}"),
            },
            other => panic!("expected Reply, got {other:?}"),
        }
    }
}
