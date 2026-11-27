//! The media library's commands on episode files: download, download what
//! is missing, download again what got lost, align the recorded state with
//! the storage, remove files.
//!
//! Decisions rest on the library's own record of the files
//! ([`EpisodeFileService`]), not on what the catalog hands along. What
//! other contexts care about goes out as events ([`MediaEvents`]); the UI
//! hears about files coming and going through [`FileStateListener`].

use crate::catalog::{Catalog, MediaEpisode, MediaPodcast};
use crate::download::Downloader;
use crate::events::{EpisodeDownloadFailed, EpisodeDownloaded};
use crate::files::{EpisodeFile, EpisodeFileService};
use common_infrastructure::config::FileHandlerType;
use common_infrastructure::error::CustomError;
use common_infrastructure::retry::do_retry;
use podfetch_storage::{FileHandleWrapper, FileRequest};
use std::collections::HashSet;
use std::sync::{Arc, LazyLock, Mutex, PoisonError};

/// Port to the event bus.
pub trait MediaEvents: Send + Sync {
    fn downloaded(&self, event: EpisodeDownloaded);
    fn download_failed(&self, event: EpisodeDownloadFailed);
}

/// Told when an episode's file became available or was removed, so the UI
/// can show it right away.
pub trait FileStateListener: Send + Sync {
    fn available(&self, episode_id: &str, podcast_id: &str);
    fn removed(&self, episode_id: &str);
}

/// Downloads running in this process, by the feed's episode id. Process
/// wide: two services must not write the same file at once.
static RUNNING: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

/// Marks a download as running until dropped.
struct Running(String);

impl Running {
    fn start(episode_id: &str) -> Option<Self> {
        let mut running = RUNNING.lock().unwrap_or_else(PoisonError::into_inner);
        running
            .insert(episode_id.to_string())
            .then(|| Self(episode_id.to_string()))
    }
}

impl Drop for Running {
    fn drop(&mut self) {
        RUNNING
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(&self.0);
    }
}

/// How many episodes of one podcast are downloaded at the same time when
/// several are asked for at once.
const PARALLEL_DOWNLOADS: usize = 3;

pub struct MediaLibrary {
    catalog: Arc<dyn Catalog>,
    downloader: Arc<Downloader>,
    files: Arc<EpisodeFileService>,
    events: Arc<dyn MediaEvents>,
    listener: Arc<dyn FileStateListener>,
}

impl MediaLibrary {
    pub fn new(
        catalog: Arc<dyn Catalog>,
        downloader: Arc<Downloader>,
        files: Arc<EpisodeFileService>,
        events: Arc<dyn MediaEvents>,
        listener: Arc<dyn FileStateListener>,
    ) -> Self {
        Self {
            catalog,
            downloader,
            files,
            events,
            listener,
        }
    }

    fn file(&self, episode: &MediaEpisode) -> Result<EpisodeFile, CustomError> {
        self.files.file(&episode.id)
    }

    /// Downloads the episode, also when its file is there already. A
    /// download of the same episode that is running already isn't started a
    /// second time. Returns whether it ran.
    pub fn download(
        &self,
        episode: &MediaEpisode,
        podcast: &MediaPodcast,
    ) -> Result<bool, CustomError> {
        let Some(_running) = Running::start(&episode.episode_id) else {
            tracing::info!(
                "Skipping download for episode {} because a download is already running",
                episode.episode_id
            );
            return Ok(false);
        };
        tracing::info!("Downloading podcast episode: {}", episode.name);
        if let Err(err) = self.downloader.download(episode, podcast) {
            self.events.download_failed(EpisodeDownloadFailed {
                episode_id: episode.episode_id.clone(),
                episode_name: episode.name.clone(),
                podcast_id: podcast.id.clone(),
                reason: err.inner.to_string(),
            });
            return Err(err);
        }
        self.files.record_download(
            &episode.id,
            Some(self.downloader.file_handler().to_string()),
            chrono::Utc::now().naive_utc(),
        )?;
        self.events.downloaded(EpisodeDownloaded {
            episode_id: episode.episode_id.clone(),
            episode_name: episode.name.clone(),
            podcast_id: podcast.id.clone(),
            podcast_name: podcast.name.clone(),
        });
        self.listener.available(&episode.id, &podcast.id);
        Ok(true)
    }

    /// Downloads the episode unless its file is there already.
    pub fn download_if_missing(
        &self,
        episode: &MediaEpisode,
        podcast: &MediaPodcast,
    ) -> Result<(), CustomError> {
        if self.file(episode)?.download_location.is_some() {
            return Ok(());
        }
        self.download(episode, podcast).map(drop)
    }

    /// A user asked for the episode: downloads it and takes back an earlier
    /// deletion, so it is kept like any other download.
    pub fn download_on_request(
        &self,
        episode: &MediaEpisode,
        podcast: &MediaPodcast,
    ) -> Result<(), CustomError> {
        self.download(episode, podcast)?;
        self.files.set_deleted(&episode.id, false)
    }

    /// Downloads every episode of the podcast that has no file and wasn't
    /// deleted by a user. Returns how many were missing.
    pub fn download_missing(&self, podcast_id: &str) -> Result<usize, CustomError> {
        let podcast = self.catalog.podcast(podcast_id)?;
        let episodes = self.catalog.episodes_of_podcast(podcast_id)?;
        self.download_missing_in(&podcast, episodes)
    }

    /// Like [`Self::download_missing`], for the episodes at the positions
    /// `from..=to` (oldest = 1, by recording date).
    pub fn download_range(
        &self,
        podcast_id: &str,
        from: usize,
        to: usize,
    ) -> Result<usize, CustomError> {
        let podcast = self.catalog.podcast(podcast_id)?;
        let mut episodes = self.catalog.episodes_of_podcast(podcast_id)?;
        episodes.sort_by(|a, b| a.date_of_recording.cmp(&b.date_of_recording));
        let start = from.saturating_sub(1).min(episodes.len());
        let end = to.min(episodes.len());
        let range = if start < end {
            episodes.drain(start..end).collect()
        } else {
            Vec::new()
        };
        self.download_missing_in(&podcast, range)
    }

    fn download_missing_in(
        &self,
        podcast: &MediaPodcast,
        episodes: Vec<MediaEpisode>,
    ) -> Result<usize, CustomError> {
        let mut missing = Vec::new();
        for episode in episodes {
            let file = self.file(&episode)?;
            if !file.deleted && file.download_location.is_none() {
                missing.push(episode);
            }
        }
        download_all(podcast, &missing, |episode| {
            self.download_if_missing(episode, podcast)
        });
        Ok(missing.len())
    }

    /// Downloads again the episodes recorded as downloaded whose file is
    /// gone from the storage. Returns how many.
    pub fn download_lost(&self, podcast_id: &str) -> Result<usize, CustomError> {
        let podcast = self.catalog.podcast(podcast_id)?;
        let mut lost = Vec::new();
        for episode in self.catalog.episodes_of_podcast(podcast_id)? {
            let file = self.file(&episode)?;
            if !file.deleted && file.download_location.is_some() && file_missing(&file) {
                lost.push(episode);
            }
        }
        download_all(&podcast, &lost, |episode| {
            self.download(episode, &podcast).map(drop)
        });
        Ok(lost.len())
    }

    /// Forgets the download of episodes whose file is gone from the
    /// storage; the storage is the truth, nothing is downloaded. Returns
    /// how many.
    pub fn resync(&self, podcast_id: &str) -> Result<usize, CustomError> {
        let mut affected = 0;
        for episode in self.catalog.episodes_of_podcast(podcast_id)? {
            let file = self.file(&episode)?;
            if file.download_location.is_some() && file_missing(&file) {
                do_retry(|| self.files.clear_download(&episode.id))?;
                self.listener.removed(&episode.id);
                affected += 1;
            }
        }
        Ok(affected)
    }

    /// Removes the episode's files from the storage and forgets the
    /// download. The episode may be downloaded again on its own.
    pub fn remove_file(&self, episode: &MediaEpisode) -> Result<(), CustomError> {
        let file = self.file(episode)?;
        let Some(location) = file.download_location.as_deref() else {
            // Never downloaded: nothing on the storage.
            return Ok(());
        };
        tracing::info!("Removing the files of episode {}", episode.episode_id);
        let handler = FileHandlerType::from_stored(location);
        for path in [&file.file_episode_path, &file.file_image_path]
            .into_iter()
            .flatten()
        {
            if FileHandleWrapper::path_exists(path, FileRequest::File, &handler) {
                FileHandleWrapper::remove_file(path, &handler)?;
            }
        }
        do_retry(|| self.files.clear_download(&episode.id))
    }

    /// A user deleted the episode's file: it is removed and not downloaded
    /// again unless a user asks for it.
    pub fn delete_on_request(&self, episode: &MediaEpisode) -> Result<(), CustomError> {
        self.remove_file(episode)?;
        self.files.set_deleted(&episode.id, true)?;
        self.listener.removed(&episode.id);
        Ok(())
    }

    /// Removes the files of every downloaded episode of the podcast but
    /// those in `keep` (catalog ids). The episodes stay. Returns how many
    /// files were removed.
    pub fn remove_files(
        &self,
        podcast_id: &str,
        keep: &HashSet<String>,
    ) -> Result<usize, CustomError> {
        let mut affected = 0;
        for episode in self.catalog.episodes_of_podcast(podcast_id)? {
            if keep.contains(&episode.id) || self.file(&episode)?.download_location.is_none() {
                continue;
            }
            self.remove_file(&episode)?;
            self.listener.removed(&episode.id);
            affected += 1;
        }
        Ok(affected)
    }
}

/// Runs `download` for the episodes, a few at a time; failures are
/// logged.
fn download_all(
    podcast: &MediaPodcast,
    episodes: &[MediaEpisode],
    download: impl Fn(&MediaEpisode) -> Result<(), CustomError> + Sync,
) {
    for chunk in episodes.chunks(PARALLEL_DOWNLOADS) {
        std::thread::scope(|scope| {
            for episode in chunk {
                let download = &download;
                scope.spawn(move || {
                    if let Err(err) = download(episode) {
                        tracing::error!(
                            "Error downloading episode {} of podcast {}: {err}",
                            episode.episode_id,
                            podcast.id
                        );
                    }
                });
            }
        });
    }
}

/// Whether a file recorded as downloaded is missing on its storage.
fn file_missing(file: &EpisodeFile) -> bool {
    let (Some(path), Some(location)) = (
        file.file_episode_path.as_deref(),
        file.download_location.as_deref(),
    ) else {
        return true;
    };
    !FileHandleWrapper::path_exists(
        path,
        FileRequest::File,
        &FileHandlerType::from_stored(location),
    )
}

#[cfg(test)]
mod tests {
    use super::Running;

    #[test]
    fn an_episode_is_downloaded_once_at_a_time() {
        let first = Running::start("library-test-episode").expect("nothing runs yet");
        assert!(Running::start("library-test-episode").is_none());
        assert!(Running::start("library-test-other").is_some());
        drop(first);
        assert!(Running::start("library-test-episode").is_some());
    }
}
