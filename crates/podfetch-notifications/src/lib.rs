//! Notifications bounded context.
//!
//! Owns the in-app notifications and the Telegram channel. It never gets
//! called by other contexts: it reacts to their events (see [`inbound`])
//! through the handlers in [`handlers`].

pub mod domain;
pub mod dto;
pub mod handlers;
pub mod inbound;
pub mod persistence;
pub mod service;
pub mod telegram;
