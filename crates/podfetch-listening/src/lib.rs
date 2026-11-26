//! Listening & progress bounded context.
//!
//! Episode actions (the gPodder action log, which is also the playback
//! progress), listening events and the statistics built from them. The
//! context knows users and episodes by id and guid only; podcast names for
//! the statistics come through the [`services::stats::PodcastLookup`] port.

pub mod domain;
pub mod dto;
pub mod handlers;
pub mod persistence;
pub mod services;
