//! Podcast catalog bounded context: the podcasts a server subscribes to,
//! their episodes and chapters, read from the feeds.
//!
//! Other contexts reach it through [`api::CatalogApi`] and react to what
//! it publishes in [`events`]. It knows nothing about them: the media
//! library, the library, listening and transcripts keep catalog ids only.
//!
//! The episode persistence still reads the media library's file state
//! through a join; replacing it with API composition is migration step
//! 5.3d. The feed sync and import services follow in 5.3c.

pub mod api;
pub mod domain;
pub mod events;
pub mod persistence;
