//! Media library bounded context: the files behind the catalog.
//!
//! Downloads, episode files and their storage, SponsorBlock segments.
//! The context keeps episode and user ids only; what an episode is (its
//! name, feed URL, YouTube video id) comes from the catalog, translated at
//! the boundary.
//!
//! So far it holds SponsorBlock; downloads, files and rescans follow
//! (migration step 5.2b).

pub mod handlers;
pub mod persistence;
pub mod sponsorblock;
