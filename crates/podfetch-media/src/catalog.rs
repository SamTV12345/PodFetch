//! What the media library knows about the catalog, and what it asks of it.
//!
//! The catalog is upstream: the media library reads podcasts and episodes
//! through [`Catalog`] and records what it learned from a file (duration,
//! chapters, numbering, cover) by calling it. The types here are the media
//! library's own view, translated from the catalog at the boundary.

use crate::audio::Chapter;
use crate::files::EpisodeFile;
use common_infrastructure::error::CustomError;

/// A catalog episode as the media library sees it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MediaEpisode {
    /// The catalog id.
    pub id: String,
    /// The id the feed gives the episode.
    pub episode_id: String,
    pub podcast_id: String,
    pub name: String,
    /// The enclosure URL.
    pub url: String,
    pub image_url: String,
    pub date_of_recording: String,
    pub description: String,
    pub guid: String,
    /// Length in seconds; 0 if unknown.
    pub total_time: i32,
    /// The title in the file already carries the episode number.
    pub episode_numbering_processed: bool,
    pub youtube_video_id: Option<String>,
    pub file: EpisodeFile,
}

/// A catalog podcast as the media library sees it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MediaPodcast {
    /// The catalog id.
    pub id: String,
    /// The feed/track id the podcast's image URL is stored by.
    pub directory_id: String,
    pub name: String,
    /// The directory the podcast's files live in.
    pub directory_name: String,
    /// The stored (local, URL-encoded) image URL.
    pub image_url: String,
    /// The image URL from the feed.
    pub original_image_url: String,
    pub author: Option<String>,
    pub keywords: Option<String>,
    pub language: Option<String>,
    pub summary: Option<String>,
    pub guid: Option<String>,
    /// The storage the podcast's files live on.
    pub download_location: Option<String>,
}

/// Port to the catalog.
pub trait Catalog: Send + Sync {
    fn podcast(&self, podcast_id: &str) -> Result<MediaPodcast, CustomError>;
    fn episodes_of_podcast(&self, podcast_id: &str) -> Result<Vec<MediaEpisode>, CustomError>;
    /// The episode's position in its podcast, by recording date.
    fn position_of_episode(
        &self,
        podcast_id: &str,
        date_of_recording: &str,
    ) -> Result<usize, CustomError>;
    fn episode_count(&self, podcast_id: &str) -> Result<usize, CustomError>;
    /// The track number of the episode recorded at `date_of_recording`.
    fn track_number(&self, podcast_id: &str, date_of_recording: &str) -> Result<i64, CustomError>;
    /// Whether the podcast's own cover is on disk already.
    fn podcast_cover_downloaded(&self, directory_id: &str) -> bool;

    /// Records whether the title in the file carries the episode number.
    fn set_numbered(&self, episode: &MediaEpisode, numbered: bool) -> Result<(), CustomError>;
    /// Records the real length of the downloaded file.
    fn set_duration(&self, episode: &MediaEpisode, seconds: i32) -> Result<(), CustomError>;
    /// Stores a chapter read from the episode's file.
    fn save_chapter(&self, episode: &MediaEpisode, chapter: &Chapter) -> Result<(), CustomError>;
    /// Points the podcast's image at a cover file the media library stored.
    fn set_podcast_image(&self, podcast: &MediaPodcast, image_url: &str)
    -> Result<(), CustomError>;
}
