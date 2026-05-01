//! WS client loop. Connects to the remote `/agent/ws`, performs the
//! Hello/HelloAck handshake, then runs the read/write loop until
//! disconnection — at which point it backs off and retries.

use crate::agent::config::{self, AgentConfig};
use crate::agent::inbound::{self, InboundOutcome};
use futures::{SinkExt, StreamExt};
use podfetch_agent_protocol::{
    AgentCapabilities, AgentMsg, PROTOCOL_VERSION, ServerMsg,
};
use podfetch_cast::DiscoveredCastDevice;
use tokio::time::sleep;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::Request;
use tokio_tungstenite::tungstenite::Message;
use tracing::{info, warn};

const AGENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Run the agent loop. Never returns under normal operation — the caller
/// should treat the future as the agent's lifetime.
pub async fn run(config: AgentConfig) -> std::io::Result<()> {
    let agent_id = config
        .agent_id
        .clone()
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    info!(%agent_id, remote = %config.remote, "agent starting");

    let url = config::ws_url(&config.remote).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string())
    })?;

    // Real mDNS discovery is Phase 5c — for now the agent advertises an
    // empty device list so the orchestrator sees the connection but
    // gracefully reports no devices.
    let devices: Vec<DiscoveredCastDevice> = Vec::new();

    let mut backoff = config.reconnect_initial;
    loop {
        match connect_and_run(&url, &config.api_key, &agent_id, &devices).await {
            Ok(()) => {
                info!(%agent_id, "agent session ended cleanly; reconnecting");
                backoff = config.reconnect_initial;
            }
            Err(err) => {
                warn!(%agent_id, "agent session ended with error: {err}; retrying in {:?}", backoff);
                sleep(backoff).await;
                backoff = (backoff * 2).min(config.reconnect_max);
            }
        }
    }
}

async fn connect_and_run(
    url: &str,
    api_key: &str,
    agent_id: &str,
    devices: &[DiscoveredCastDevice],
) -> Result<(), AgentRunError> {
    let request = build_request(url, api_key)?;
    let (ws_stream, _resp) = tokio_tungstenite::connect_async(request).await?;
    info!(agent_id, "connected to remote");

    let (mut sink, mut stream) = ws_stream.split();

    // Wait for the server's Hello.
    match next_server_msg(&mut stream).await? {
        Some(ServerMsg::Hello {
            protocol_version,
            server_version,
        }) => {
            if protocol_version != PROTOCOL_VERSION {
                return Err(AgentRunError::ProtocolMismatch {
                    server: protocol_version,
                    agent: PROTOCOL_VERSION,
                });
            }
            info!(
                agent_id,
                server_version = %server_version,
                "handshake: server hello received"
            );
        }
        Some(other) => return Err(AgentRunError::UnexpectedHandshake(format!("{other:?}"))),
        None => return Err(AgentRunError::ClosedDuringHandshake),
    }

    // Reply with HelloAck.
    let hello_ack = AgentMsg::HelloAck {
        protocol_version: PROTOCOL_VERSION,
        agent_id: agent_id.to_string(),
        agent_version: AGENT_VERSION.to_string(),
        capabilities: AgentCapabilities {
            chromecast: true,
            local_proxy: false,
        },
    };
    send_msg(&mut sink, &hello_ack).await?;

    // Push the (currently empty) device list so the server sees us as a
    // healthy, reporting agent.
    let initial_list = AgentMsg::DeviceList {
        request_id: None,
        devices: devices.to_vec(),
    };
    send_msg(&mut sink, &initial_list).await?;

    // Main loop: process inbound messages until the stream closes.
    loop {
        match next_server_msg(&mut stream).await? {
            None => return Ok(()),
            Some(msg) => match inbound::dispatch(msg, devices) {
                InboundOutcome::Reply(replies) => {
                    for reply in replies {
                        send_msg(&mut sink, &reply).await?;
                    }
                }
                InboundOutcome::Goodbye => return Ok(()),
                InboundOutcome::Ignore => {}
            },
        }
    }
}

fn build_request(url: &str, api_key: &str) -> Result<Request<()>, AgentRunError> {
    let mut request = url
        .into_client_request()
        .map_err(|e| AgentRunError::BadUrl(e.to_string()))?;
    let header_value = format!("Bearer {api_key}")
        .parse()
        .map_err(|e: tokio_tungstenite::tungstenite::http::header::InvalidHeaderValue| {
            AgentRunError::BadUrl(e.to_string())
        })?;
    request.headers_mut().insert("Authorization", header_value);
    Ok(request)
}

async fn next_server_msg<S>(stream: &mut S) -> Result<Option<ServerMsg>, AgentRunError>
where
    S: futures::Stream<
            Item = Result<Message, tokio_tungstenite::tungstenite::Error>,
        > + Unpin,
{
    while let Some(frame) = stream.next().await {
        let frame = frame?;
        match frame {
            Message::Text(text) => {
                let msg: ServerMsg = serde_json::from_str(&text).map_err(|e| {
                    AgentRunError::BadFrame(format!("json: {e}; raw: {text}"))
                })?;
                return Ok(Some(msg));
            }
            Message::Binary(_) => {
                warn!("agent: binary frame from server ignored");
            }
            Message::Ping(_) | Message::Pong(_) | Message::Frame(_) => {}
            Message::Close(_) => return Ok(None),
        }
    }
    Ok(None)
}

async fn send_msg<S>(sink: &mut S, msg: &AgentMsg) -> Result<(), AgentRunError>
where
    S: SinkExt<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin,
{
    let json = serde_json::to_string(msg)
        .map_err(|e| AgentRunError::BadFrame(format!("serialize: {e}")))?;
    sink.send(Message::Text(json.into())).await?;
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum AgentRunError {
    #[error("websocket error: {0}")]
    Ws(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("invalid request URL: {0}")]
    BadUrl(String),
    #[error("malformed frame: {0}")]
    BadFrame(String),
    #[error("server closed during handshake")]
    ClosedDuringHandshake,
    #[error("expected Hello, got {0}")]
    UnexpectedHandshake(String),
    #[error("protocol mismatch (server={server}, agent={agent})")]
    ProtocolMismatch { server: u32, agent: u32 },
}

impl From<AgentRunError> for std::io::Error {
    fn from(value: AgentRunError) -> Self {
        std::io::Error::other(value.to_string())
    }
}
