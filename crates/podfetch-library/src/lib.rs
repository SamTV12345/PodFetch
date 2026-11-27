//! Library bounded context: what a user keeps and organizes.
//!
//! Favorite podcasts and episodes, tags, playlists, the saved podcast
//! filter and the episode triage (inbox / waiting list / archive). The
//! context stores podcast and episode ids only; lists that show catalog
//! data together with library data are composed by the web layer.

pub mod domain;
pub mod dto;
pub mod events;
pub mod handlers;
pub mod persistence;
pub mod services;
