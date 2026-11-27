//! Media library bounded context: the files behind the catalog.
//!
//! Downloads, episode files and their storage, SponsorBlock segments.
//! The context keeps episode and user ids only; what an episode is (its
//! name, feed URL, YouTube video id) comes from the catalog, translated at
//! the boundary.
//!
//! So far it holds the episodes' file state and SponsorBlock; the
//! download pipeline, rescans and NFO files follow (migration step 5.2c).

pub mod files;
pub mod handlers;
pub mod persistence;
pub mod sponsorblock;
