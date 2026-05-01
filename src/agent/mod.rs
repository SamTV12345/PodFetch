//! `--agent` mode runtime. Runs PodFetch as a thin LAN-side helper that
//! relays Chromecast control commands from a remote PodFetch instance to
//! Chromecasts on the local network.
//!
//! v1 scope (this crate-internal module):
//! - WS client with Hello/HelloAck handshake and reconnect
//! - Inbound dispatch for `DiscoverRequest`, `Play`, `Control`
//! - Pong response to Ping
//!
//! Out of scope here (planned for a follow-up): real mDNS discovery and
//! the actual Chromecast CAST protocol implementation. Until those land,
//! the agent reports a synthetic device list and replies to `Play` /
//! `Control` with `Error { code: NotImplemented }` so the rest of the
//! relay stack can be exercised end-to-end.

pub mod client;
pub mod config;
pub mod inbound;

pub use client::run as run_agent;
#[allow(unused_imports)]
pub use config::AgentConfig;
