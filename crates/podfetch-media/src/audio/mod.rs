//! Working with episode audio files: chapters, tags, conversion.

pub mod chapters;
pub mod convert;
pub mod tags;

pub use chapters::{Chapter, Link};
pub use tags::{EpisodeTags, TagFormat};
