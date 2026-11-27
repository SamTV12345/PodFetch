//! Media library bounded context: the files behind the catalog.
//!
//! Downloads, episode files and their storage, SponsorBlock segments.
//! The context keeps episode and user ids only; what an episode is (its
//! name, feed URL, YouTube video id) comes from the catalog, translated at
//! the boundary.
//!
//! So far it holds the episodes' file state, SponsorBlock and the building
//! blocks of the download pipeline (tags, chapters, conversion, naming);
//! the download, file, rescan and NFO services follow (migration step
//! 5.2c-2).

pub mod audio;
pub mod files;
pub mod handlers;
pub mod naming;
pub mod persistence;
pub mod rules;
pub mod sponsorblock;
