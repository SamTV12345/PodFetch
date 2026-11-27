//! The file state of each catalog episode: whether and where it was
//! downloaded, and whether the user deleted the file on purpose.

pub mod domain;
pub mod service;

pub use domain::{EpisodeFile, EpisodeFileRepository};
pub use service::EpisodeFileService;
