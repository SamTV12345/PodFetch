//! Podcast catalog bounded context: the podcasts a server subscribes to,
//! their episodes and chapters, read from the feeds.
//!
//! Other contexts reach it through [`api::CatalogApi`] and react to what
//! it publishes in [`events`]. It knows nothing about them: the media
//! library, the library, listening and transcripts keep catalog ids only.
//!
//! The episode persistence still reads the media library's file state
//! through a join; replacing it with API composition is migration step
//! 5.3d. Subscribing/importing podcasts and the episode use case follow
//! (migration steps 5.3c-2 and 5.3c-3).

pub mod api;
pub mod domain;
pub mod events;
pub mod feed;
pub mod persistence;
