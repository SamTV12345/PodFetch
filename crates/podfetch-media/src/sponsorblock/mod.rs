//! SponsorBlock: community-submitted segments (sponsors, intros, …) of
//! YouTube videos, fetched when an episode is downloaded, and each user's
//! choice of which segments the player skips.

pub mod client;
pub mod domain;
pub mod service;

pub use client::SponsorblockClient;
pub use domain::{SkipPreferences, SponsorSegment, SponsorblockRepository};
pub use service::{EpisodeToCheck, SponsorblockService, SponsorblockSwitch};
