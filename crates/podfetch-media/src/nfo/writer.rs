//! Writes NFO files and keeps the podcast cover under its configured name.
//! All writes are best-effort: failures are logged, never propagated (they
//! must not fail a download or a settings save).

use super::{NfoFormat, builders};
use crate::catalog::{Catalog, MediaEpisode, MediaPodcast};
use crate::settings::MediaSettingsSource;
use common_infrastructure::config::FileHandlerType;
use podfetch_storage::{FileHandleWrapper, FileRequest};
use std::sync::Arc;

/// File names a podcast cover may have.
pub const COVER_CANDIDATES: &[&str] = &["image", "cover", "folder", "poster"];

pub struct NfoWriter {
    catalog: Arc<dyn Catalog>,
    settings: Arc<dyn MediaSettingsSource>,
    file_handler: FileHandlerType,
}

impl NfoWriter {
    pub fn new(
        catalog: Arc<dyn Catalog>,
        settings: Arc<dyn MediaSettingsSource>,
        file_handler: FileHandlerType,
    ) -> Self {
        Self {
            catalog,
            settings,
            file_handler,
        }
    }

    /// Writes the NFO of one episode (and refreshes the podcast-level NFO).
    /// `audio_path` is the FINAL media path (post-transcode), so the
    /// per-episode `.nfo` basename matches the audio file.
    pub fn regenerate_for_episode(
        &self,
        podcast: &MediaPodcast,
        episode: &MediaEpisode,
        audio_path: &str,
    ) {
        let format = match self.settings.for_podcast(&podcast.id) {
            Ok(settings) => settings.nfo_format,
            Err(err) => {
                tracing::warn!("NFO: could not load the settings: {err}");
                return;
            }
        };
        match format {
            NfoFormat::Off => {}
            NfoFormat::Tvshow => {
                let position = self
                    .catalog
                    .position_of_episode(&podcast.id, &episode.date_of_recording)
                    .unwrap_or(0) as i64;
                self.write(
                    &nfo_path_for(audio_path),
                    &builders::build_episodedetails_nfo(podcast, episode, position),
                );
                self.write(
                    &format!("{}/tvshow.nfo", podcast.directory_name),
                    &builders::build_tvshow_nfo(podcast),
                );
            }
            NfoFormat::Album => self.write_album_nfo(podcast),
        }
    }

    /// Rewrites `album.nfo` from the podcast's downloaded episodes.
    fn write_album_nfo(&self, podcast: &MediaPodcast) {
        let mut episodes = match self.catalog.episodes_of_podcast(&podcast.id) {
            Ok(e) => e,
            Err(err) => {
                tracing::warn!("album.nfo: could not load episodes: {err}");
                return;
            }
        };
        episodes.retain(|e| e.file.download_time.is_some());
        episodes.sort_by(|a, b| a.date_of_recording.cmp(&b.date_of_recording));
        let tracks: Vec<(MediaEpisode, i64)> = episodes
            .into_iter()
            .enumerate()
            .map(|(i, e)| (e, (i as i64) + 1))
            .collect();
        self.write(
            &format!("{}/album.nfo", podcast.directory_name),
            &builders::build_album_nfo(podcast, &tracks),
        );
    }

    /// Renames the on-disk podcast cover to the configured base name when
    /// needed. Local storage only (rename is unsupported on S3).
    pub fn ensure_cover_filename(&self, podcast: &MediaPodcast) {
        let target = match self.settings.for_podcast(&podcast.id) {
            Ok(settings) => settings.cover_filename,
            Err(err) => {
                tracing::warn!("Cover rename: could not load the settings: {err}");
                return;
            }
        };
        let dir = &podcast.directory_name;
        if dir.is_empty() {
            return;
        }
        let Some(ext) = std::path::Path::new(&podcast.image_url)
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase)
        else {
            return;
        };

        let fh = &self.file_handler;
        let target_path = format!("{dir}/{target}.{ext}");
        if FileHandleWrapper::path_exists(&target_path, FileRequest::File, fh) {
            return; // already correct
        }
        for cand in COVER_CANDIDATES {
            if *cand == target {
                continue;
            }
            let cand_path = format!("{dir}/{cand}.{ext}");
            if !FileHandleWrapper::path_exists(&cand_path, FileRequest::File, fh) {
                continue;
            }
            match FileHandleWrapper::rename_file(&cand_path, &target_path, fh) {
                Ok(()) => {
                    tracing::info!("Renamed cover {cand_path} -> {target_path}");
                    let new_url = format!("{}/{}.{}", local_url(dir), target, ext);
                    if let Err(err) = self.catalog.set_podcast_image(podcast, &new_url) {
                        tracing::warn!("Cover renamed but DB image_url update failed: {err}");
                    }
                }
                Err(err) => {
                    tracing::warn!("Cover rename {cand_path} -> {target_path} failed: {err}");
                }
            }
            return;
        }
    }

    fn write(&self, path: &str, xml: &str) {
        // write_file requires &mut [u8]; the Vec allocation is forced by the storage API.
        let mut bytes = xml.as_bytes().to_vec();
        if let Err(err) =
            FileHandleWrapper::write_file(path, bytes.as_mut_slice(), &self.file_handler)
        {
            tracing::warn!("Failed to write NFO file {path}: {err}");
        }
    }
}

/// The URL a stored file is served under: its last path segment
/// percent-encoded.
pub fn local_url(path: &str) -> String {
    match path.rsplit_once('/') {
        Some((base, file)) => format!("{base}/{}", urlencoding::encode(file)),
        None => urlencoding::encode(path).into_owned(),
    }
}

/// Derives the sidecar `.nfo` path from a media file path by swapping its
/// extension. If the path has no extension, appends `.nfo`.
pub fn nfo_path_for(audio_path: &str) -> String {
    let last_sep = audio_path.rfind(['/', '\\']);
    match audio_path.rfind('.') {
        Some(dot) if last_sep.is_none_or(|sep| dot > sep) => {
            format!("{}.nfo", &audio_path[..dot])
        }
        _ => format!("{audio_path}.nfo"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nfo_path_swaps_extension_or_appends() {
        assert_eq!(nfo_path_for("podcasts/x/audio.mp3"), "podcasts/x/audio.nfo");
        assert_eq!(
            nfo_path_for("podcasts/My Show/2023-09-07 - Ep.mp3"),
            "podcasts/My Show/2023-09-07 - Ep.nfo"
        );
        // dot only in a directory component -> treat as no extension
        assert_eq!(
            nfo_path_for("podcasts/My.Show/episode"),
            "podcasts/My.Show/episode.nfo"
        );
        // dot in both a directory component and the filename
        assert_eq!(
            nfo_path_for("podcasts/My.Show/2024-01-01 - Ep.mp3"),
            "podcasts/My.Show/2024-01-01 - Ep.nfo"
        );
        // windows separators
        assert_eq!(
            nfo_path_for("podcasts\\x\\audio.opus"),
            "podcasts\\x\\audio.nfo"
        );
    }

    #[test]
    fn local_url_encodes_the_last_segment() {
        assert_eq!(local_url("podcasts/My Show"), "podcasts/My%20Show");
        assert_eq!(local_url("My Show"), "My%20Show");
    }
}
