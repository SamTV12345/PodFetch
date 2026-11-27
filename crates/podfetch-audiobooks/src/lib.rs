//! Audiobooks bounded context: the Audiobookshelf-compatible library of
//! books, series, authors and narrators, playback sessions and progress.
//!
//! Users come from identity through the [`account`] translation; the
//! Audiobookshelf HTTP and socket API in the web layer is the open host
//! service of this context.

pub mod account;
pub mod domain;
pub mod handlers;
pub mod persistence;
pub mod services;
