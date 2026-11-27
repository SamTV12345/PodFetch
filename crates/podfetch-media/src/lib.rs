//! Media library bounded context: the files behind the catalog.
//!
//! Downloads, episode files and their storage, SponsorBlock segments.
//! The context keeps episode and user ids only; what an episode is (its
//! name, feed URL, YouTube video id) comes from the catalog, translated at
//! the boundary.
//!
//! It reaches the catalog, the settings and the transcripts only through
//! the ports in [`catalog`], [`settings`] and [`download`].

pub mod audio;
pub mod auto_download;
pub mod catalog;
pub mod download;
pub mod events;
pub mod files;
pub mod handlers;
pub mod images;
pub mod inbound;
pub mod naming;
pub mod nfo;
pub mod persistence;
pub mod rescan;
pub mod rules;
pub mod settings;
pub mod sponsorblock;
