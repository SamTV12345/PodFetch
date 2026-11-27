//! The settings the media library works by, resolved for one podcast.

use crate::naming::NamingRules;
use crate::nfo::NfoFormat;
use common_infrastructure::error::CustomError;

/// The download, naming and file rules in effect for one podcast: the
/// podcast's own settings where they override the instance-wide ones.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MediaSettings {
    pub naming: NamingRules,
    /// Name episode files after the enclosure's file name.
    pub use_existing_filename: bool,
    /// Put episode files straight into the podcast directory.
    pub direct_paths: bool,
    pub auto_transcode_opus: bool,
    /// Keep only the podcast's cover, no per-episode images.
    pub use_one_cover: bool,
    /// Prefix titles in the files with the episode number.
    pub numbering: bool,
    /// Base name of the podcast's cover file.
    pub cover_filename: String,
    pub nfo_format: NfoFormat,
    /// Generate a transcript for each downloaded episode.
    pub auto_transcribe: bool,
}

/// Port to the settings.
pub trait MediaSettingsSource: Send + Sync {
    fn for_podcast(&self, podcast_id: &str) -> Result<MediaSettings, CustomError>;
}
