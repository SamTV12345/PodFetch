//! Casting bounded context: playback on Chromecast and Mopidy devices,
//! either on the server's own network or through a remote agent.
//!
//! [`orchestrator::CastOrchestrator`] is the entry point. It owns the cast
//! targets ([`devices`]) and the active sessions, and reacts to identity
//! events through [`handlers`].

pub mod agent;
pub mod devices;
pub mod dto;
pub mod handlers;
pub mod mopidy;
pub mod orchestrator;
pub mod persistence;
