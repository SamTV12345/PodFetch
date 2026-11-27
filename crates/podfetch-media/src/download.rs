//! Downloading an episode: fetch the audio (and its image), name and store
//! the files, tag them, and record what the file tells about the episode.

use crate::audio::{Chapter, EpisodeTags, TagFormat};
use crate::catalog::{Catalog, MediaEpisode, MediaPodcast};
use crate::files::EpisodeFileService;
use crate::naming::EpisodeNaming;
use crate::nfo::NfoWriter;
use crate::settings::{MediaSettings, MediaSettingsSource};
use crate::sponsorblock::{EpisodeToCheck, SponsorblockService};
use common_infrastructure::config::{EnvironmentService, FileHandlerType};
use common_infrastructure::error::{CustomError, map_reqwest_error};
use common_infrastructure::http::{COMMON_USER_AGENT, get_sync_client};
use common_infrastructure::runtime::{PODCAST_FILENAME, is_default_image_url};
use podfetch_storage::{
    FileHandleWrapper, FileRequest, FileType, FilenameBuilder, FilenameBuilderReturn,
    create_available_directory, determine_file_extension, resolve_file_handler_type,
};
use reqwest::header::{ACCEPT_ENCODING, HeaderMap, HeaderValue, USER_AGENT};
use std::path::Path;
use std::sync::Arc;

/// Told about every downloaded episode, before it is transcoded (the
/// transcripts context reads the original file).
pub trait DownloadListener: Send + Sync {
    fn downloaded(&self, episode: &MediaEpisode, file_path: &str, settings: &MediaSettings);
}

/// Measures the real length of an audio file.
pub trait DurationProbe: Send + Sync {
    /// Length in seconds.
    fn duration(&self, file_path: &str) -> Result<f64, CustomError>;
}

pub struct Downloader {
    pub(crate) environment: Arc<EnvironmentService>,
    pub(crate) catalog: Arc<dyn Catalog>,
    pub(crate) settings: Arc<dyn MediaSettingsSource>,
    pub(crate) files: Arc<EpisodeFileService>,
    pub(crate) sponsorblock: Arc<SponsorblockService>,
    pub(crate) nfo: Arc<NfoWriter>,
    pub(crate) listener: Arc<dyn DownloadListener>,
    pub(crate) probe: Arc<dyn DurationProbe>,
}

impl Downloader {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        environment: Arc<EnvironmentService>,
        catalog: Arc<dyn Catalog>,
        settings: Arc<dyn MediaSettingsSource>,
        files: Arc<EpisodeFileService>,
        sponsorblock: Arc<SponsorblockService>,
        nfo: Arc<NfoWriter>,
        listener: Arc<dyn DownloadListener>,
        probe: Arc<dyn DurationProbe>,
    ) -> Self {
        Self {
            environment,
            catalog,
            settings,
            files,
            sponsorblock,
            nfo,
            listener,
            probe,
        }
    }

    fn file_handler(&self) -> &FileHandlerType {
        &self.environment.default_file_handler
    }

    /// Downloads `episode` of `podcast` and records its files. Steps after
    /// storing the audio (tags, chapters, transcripts, SponsorBlock,
    /// duration, Opus, NFO) only log their failures.
    #[tracing::instrument(skip_all, fields(
        episode_id = episode.id,
        episode_name = %episode.name,
        podcast_id = podcast.id,
    ))]
    pub fn download(
        &self,
        episode: &MediaEpisode,
        podcast: &MediaPodcast,
    ) -> Result<(), CustomError> {
        let mut header_map = HeaderMap::new();
        header_map.insert(USER_AGENT, HeaderValue::from_static(COMMON_USER_AGENT));
        header_map.insert(ACCEPT_ENCODING, HeaderValue::from_static("identity"));
        let client = get_sync_client(&self.environment)
            .default_headers(header_map)
            .no_gzip()
            .no_brotli()
            .no_deflate()
            .timeout(None)
            .no_zstd()
            .build()
            .map_err(map_reqwest_error)?;
        let mut podcast_data = crate::images::fetch_with_suffix(
            &self.environment,
            determine_file_extension(&episode.url, &client, FileType::Audio),
            &episode.url,
        )?;
        if crate::audio::convert::is_hls_playlist(&podcast_data.1) {
            tracing::info!(
                "Episode {} is served as an HLS playlist, downloading via ffmpeg",
                episode.episode_id
            );
            podcast_data = crate::audio::convert::download_hls_episode(&episode.url)?;
        }
        let settings = self.settings.for_podcast(&podcast.id)?;
        let episode_has_own_image = !is_default_image_url(&episode.image_url);
        let podcast_main_image_downloaded =
            self.catalog.podcast_cover_downloaded(&podcast.directory_id);

        // Download when: episode has its own image, or the podcast main image
        // hasn't been downloaded yet (and we're not forced to use one cover).
        let should_download_main_image = crate::rules::should_download_image(
            settings.use_one_cover,
            episode_has_own_image,
            podcast_main_image_downloaded,
        );

        let mut image_data = if should_download_main_image {
            let image_url = if episode_has_own_image {
                Some(episode.image_url.as_str())
            } else if !is_default_image_url(&podcast.original_image_url) {
                tracing::info!(
                    "Episode {} has no image, falling back to podcast image",
                    episode.episode_id
                );
                Some(podcast.original_image_url.as_str())
            } else {
                None
            };

            match image_url {
                Some(url) => Some(crate::images::fetch_with_suffix(
                    &self.environment,
                    determine_file_extension(url, &client, FileType::Image),
                    url,
                )?),
                None => Some(crate::images::default_image()),
            }
        } else {
            None
        };

        let episode_stem = self.episode_stem(episode, &settings)?;
        let paths = FilenameBuilder::default()
            .with_podcast_directory(&podcast.directory_name)
            .with_episode_stem(&episode_stem)
            .with_suffix(&podcast_data.0)
            .with_image_suffix(
                &image_data
                    .as_ref()
                    .map_or_else(|| "jpg".to_string(), |data| data.0.clone()),
            )
            .with_image_filename(&settings.cover_filename)
            .with_filename(PODCAST_FILENAME)
            .with_direct_paths(settings.direct_paths)
            .build(|directory| episode_directory_available(&directory, podcast))?;

        if !FileHandleWrapper::path_exists(
            &podcast.directory_name,
            FileRequest::Directory,
            self.file_handler(),
        ) {
            FileHandleWrapper::create_dir(&podcast.directory_name, self.file_handler())?;
        }

        // `paths.filename` is a `String`, so its parent is valid UTF-8 too.
        if let Some(parent) = Path::new(&paths.filename).parent().and_then(Path::to_str)
            && !FileHandleWrapper::path_exists(parent, FileRequest::Directory, self.file_handler())
        {
            FileHandleWrapper::create_dir(parent, self.file_handler())?;
        }

        let image_written =
            should_download_main_image && image_data.as_ref().is_some_and(|d| !d.1.is_empty());
        if image_written && let Some(ref mut data) = image_data {
            FileHandleWrapper::write_file(
                &paths.image_filename,
                data.1.as_mut_slice(),
                self.file_handler(),
            )?;
        }

        FileHandleWrapper::write_file(
            &paths.filename,
            podcast_data.1.as_mut_slice(),
            self.file_handler(),
        )?;

        // Read chapters and embed tags BEFORE transcoding. Opus has no
        // id3/mp4-atom metadata, and transcoding would also delete the
        // source file we need to read chapters from. Doing this first means
        // chapters are preserved in the DB even when the source is removed
        // by the transcode step below.
        match self.tag_file(&paths, episode, podcast, &settings) {
            Ok(chapters) => {
                tracing::info!("Inserting chapters for episode {}", episode.id);
                for chapter in &chapters {
                    if let Err(err) = self.catalog.save_chapter(episode, chapter) {
                        tracing::error!(
                            "Error while saving chapter for episode {}: {}",
                            episode.id,
                            err
                        );
                    }
                }
            }
            Err(err) => tracing::error!("Error handling metadata insertion: {err:?}"),
        }

        // Transcripts (feed transcripts, auto-transcription). Must never fail
        // the download.
        self.listener
            .downloaded(episode, &paths.filename, &settings);

        // SponsorBlock: fetch + store segments for YouTube-sourced episodes.
        // Non-fatal — must never fail the download.
        match self.fetch_sponsor_segments(episode, episode.youtube_video_id.clone()) {
            Ok(n) if n > 0 => {
                tracing::info!(
                    "Stored {n} SponsorBlock segments for episode {}",
                    episode.id
                );
            }
            Ok(_) => {}
            Err(err) => {
                tracing::warn!(
                    "SponsorBlock fetch failed for episode {}: {err}",
                    episode.id
                );
            }
        }

        // The feed's itunes:duration can be shorter than the file (dynamic ad
        // insertion), so prefer the real duration. Non-fatal — must never fail
        // the download.
        self.sync_duration_with_file(episode, &paths.filename);

        let final_episode_path = if settings.auto_transcode_opus {
            match crate::audio::convert::transcode_to_opus(&paths.filename) {
                Ok(opus_path) => opus_path,
                Err(e) => {
                    tracing::warn!("Opus transcoding failed, keeping original file: {e}");
                    paths.filename.clone()
                }
            }
        } else {
            paths.filename.clone()
        };

        // Jellyfin/Kodi NFO sidecar files (non-fatal). Uses the FINAL media path
        // so the per-episode .nfo basename matches the audio file even after
        // Opus transcoding.
        self.nfo
            .regenerate_for_episode(podcast, episode, &final_episode_path);

        self.files.set_paths(
            &episode.id,
            image_written.then_some(paths.image_filename.as_str()),
            &final_episode_path,
        )
    }

    /// Rewrites the tags of an episode downloaded earlier (after its
    /// settings changed) and stores the chapters found in its file.
    pub fn retag(&self, episode: &MediaEpisode, podcast: &MediaPodcast) -> Result<(), CustomError> {
        let settings = self.settings.for_podcast(&podcast.id)?;
        let paths = FilenameBuilderReturn::new(
            episode.file.file_episode_path.clone().unwrap_or_default(),
            episode.file.file_image_path.clone().unwrap_or_default(),
        );
        for chapter in self.tag_file(&paths, episode, podcast, &settings)? {
            if let Err(err) = self.catalog.save_chapter(episode, &chapter) {
                tracing::error!(
                    "Error while saving chapter for episode {}: {}",
                    episode.id,
                    err
                );
            }
        }
        Ok(())
    }

    /// Tags the downloaded file with the episode's metadata and returns the
    /// chapters embedded in it. Files on S3 are left alone. A failure to
    /// tag is logged; only an unreadable or unsupported file is an error.
    pub fn tag_file(
        &self,
        paths: &FilenameBuilderReturn,
        episode: &MediaEpisode,
        podcast: &MediaPodcast,
        settings: &MediaSettings,
    ) -> Result<Vec<Chapter>, CustomError> {
        if *self.file_handler() == FileHandlerType::S3 {
            return Ok(vec![]);
        }

        let format = TagFormat::of_file(&paths.filename)?;
        let chapters = format.read_chapters(&paths.filename)?;
        let written = self
            .episode_tags(episode, podcast, settings)
            .and_then(|tags| {
                format.write(
                    &paths.filename,
                    &paths.image_filename,
                    &tags,
                    &mut |numbered| self.catalog.set_numbered(episode, numbered),
                )
            });
        if let Err(err) = written {
            tracing::error!("Error updating metadata: {err:?}");
        }
        Ok(chapters)
    }

    /// What goes into the file's tags.
    fn episode_tags(
        &self,
        episode: &MediaEpisode,
        podcast: &MediaPodcast,
        settings: &MediaSettings,
    ) -> Result<EpisodeTags, CustomError> {
        let index = self
            .catalog
            .position_of_episode(&episode.podcast_id, &episode.date_of_recording)?;
        let track_number = self
            .catalog
            .track_number(&podcast.id, &episode.date_of_recording)
            .inspect_err(|e| tracing::error!("Error getting track number: {e:?}"))
            .ok();
        Ok(EpisodeTags {
            title: crate::rules::resolve_episode_title(
                &episode.name,
                episode.episode_numbering_processed,
                settings.numbering,
                index,
            ),
            artist: podcast.author.clone(),
            album: podcast.name.clone(),
            genre: podcast.keywords.clone(),
            language: podcast.language.clone(),
            comment: episode.description.clone(),
            date_of_recording: episode.date_of_recording.clone(),
            track_number,
        })
    }

    /// The name of the episode's file (and directory), without extension.
    pub fn episode_stem(
        &self,
        episode: &MediaEpisode,
        settings: &MediaSettings,
    ) -> Result<String, CustomError> {
        if settings.use_existing_filename
            && let Some(filename) = crate::naming::filename_of_url(&episode.url)
        {
            return Ok(filename);
        }
        let episode_number = self
            .catalog
            .position_of_episode(&episode.podcast_id, &episode.date_of_recording)?
            as i64;
        let total_episode_count = self.catalog.episode_count(&episode.podcast_id)?;
        crate::naming::episode_file_stem(
            &settings.naming,
            &EpisodeNaming {
                name: episode.name.clone(),
                date_of_recording: episode.date_of_recording.clone(),
                guid: episode.guid.clone(),
                url: episode.url.clone(),
                description: episode.description.clone(),
                total_time: episode.total_time,
            },
            episode_number,
            Some(total_episode_count),
        )
    }

    /// Fetches and stores the episode's SponsorBlock segments from
    /// synchronous code, whatever async context the caller runs in.
    pub(crate) fn fetch_sponsor_segments(
        &self,
        episode: &MediaEpisode,
        youtube_video_id: Option<String>,
    ) -> Result<usize, CustomError> {
        let sponsorblock = self.sponsorblock.clone();
        let episode_id = episode.id.clone();
        let duration_secs = i64::from(episode.total_time);
        common_infrastructure::background::run_isolated("sponsorblock-fetch", move || async move {
            sponsorblock
                .fetch_and_store(EpisodeToCheck {
                    episode_id: &episode_id,
                    youtube_video_id: youtube_video_id.as_deref(),
                    duration_secs,
                })
                .await
        })
    }

    fn sync_duration_with_file(&self, episode: &MediaEpisode, file_path: &str) {
        let duration = match self.probe.duration(file_path) {
            Ok(duration) => duration,
            Err(err) => {
                tracing::debug!("Could not probe duration of episode {}: {err}", episode.id);
                return;
            }
        };
        if let Some(total_time) = crate::rules::duration_to_store(episode.total_time, duration)
            && let Err(err) = self.catalog.set_duration(episode, total_time)
        {
            tracing::warn!("Could not update duration of episode {}: {err}", episode.id);
        }
    }
}

/// A directory for the episode's files that doesn't exist yet: `base_path`,
/// or `base_path-<n>`.
pub(crate) fn episode_directory_available(
    base_path: &str,
    podcast: &MediaPodcast,
) -> Result<String, CustomError> {
    let download_location = resolve_file_handler_type(podcast.download_location.clone());

    create_available_directory(
        base_path,
        |path| FileHandleWrapper::path_exists(path, FileRequest::Directory, &download_location),
        |path| FileHandleWrapper::path_exists(path, FileRequest::NoopS3, &download_location),
        |path| FileHandleWrapper::create_dir(path, &download_location),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::{EpisodeFile, EpisodeFileRepository};
    use crate::naming::NamingRules;
    use crate::sponsorblock::{
        SkipPreferences, SponsorSegment, SponsorblockClient, SponsorblockRepository,
        SponsorblockSwitch,
    };
    use podfetch_kernel::UserId;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // ── Fakes ────────────────────────────────────────────────────────────

    #[derive(Default)]
    struct FakeCatalog {
        numbered: Mutex<Vec<bool>>,
        durations: Mutex<Vec<i32>>,
        chapters: Mutex<usize>,
    }

    impl Catalog for FakeCatalog {
        fn podcast(&self, _: &str) -> Result<MediaPodcast, CustomError> {
            unreachable!("the download gets the podcast passed in")
        }
        fn episodes_of_podcast(&self, _: &str) -> Result<Vec<MediaEpisode>, CustomError> {
            Ok(vec![])
        }
        fn latest_episodes(&self, _: &str, _: i32) -> Result<Vec<MediaEpisode>, CustomError> {
            Ok(vec![])
        }
        fn position_of_episode(&self, _: &str, _: &str) -> Result<usize, CustomError> {
            Ok(3)
        }
        fn episode_count(&self, _: &str) -> Result<usize, CustomError> {
            Ok(10)
        }
        fn track_number(&self, _: &str, _: &str) -> Result<i64, CustomError> {
            Ok(3)
        }
        fn podcast_cover_downloaded(&self, _: &str) -> bool {
            false
        }
        fn set_numbered(&self, _: &MediaEpisode, numbered: bool) -> Result<(), CustomError> {
            self.numbered.lock().unwrap().push(numbered);
            Ok(())
        }
        fn set_duration(&self, _: &MediaEpisode, seconds: i32) -> Result<(), CustomError> {
            self.durations.lock().unwrap().push(seconds);
            Ok(())
        }
        fn save_chapter(&self, _: &MediaEpisode, _: &Chapter) -> Result<(), CustomError> {
            *self.chapters.lock().unwrap() += 1;
            Ok(())
        }
        fn set_podcast_image(&self, _: &MediaPodcast, _: &str) -> Result<(), CustomError> {
            Ok(())
        }
    }

    struct FixedSettings(MediaSettings);

    impl MediaSettingsSource for FixedSettings {
        fn for_podcast(&self, _: &str) -> Result<MediaSettings, CustomError> {
            Ok(self.0.clone())
        }
        fn max_parallel_downloads(&self) -> Result<usize, CustomError> {
            Ok(1)
        }
    }

    /// Remembers the downloaded file and whether it existed when told.
    #[derive(Default)]
    struct RecordingListener(Mutex<Vec<(String, bool)>>);

    impl DownloadListener for RecordingListener {
        fn downloaded(&self, _: &MediaEpisode, file_path: &str, _: &MediaSettings) {
            self.0
                .lock()
                .unwrap()
                .push((file_path.to_string(), Path::new(file_path).exists()));
        }
    }

    struct FixedDuration(f64);

    impl DurationProbe for FixedDuration {
        fn duration(&self, _: &str) -> Result<f64, CustomError> {
            Ok(self.0)
        }
    }

    #[derive(Default)]
    struct MemoryFiles(Mutex<HashMap<String, EpisodeFile>>);

    impl EpisodeFileRepository for MemoryFiles {
        fn find(&self, episode_id: &str) -> Result<Option<EpisodeFile>, CustomError> {
            Ok(self.0.lock().unwrap().get(episode_id).cloned())
        }
        fn modify(
            &self,
            episode_id: &str,
            change: &mut dyn FnMut(&mut EpisodeFile),
        ) -> Result<(), CustomError> {
            let mut files = self.0.lock().unwrap();
            let file = files
                .entry(episode_id.to_string())
                .or_insert_with(|| EpisodeFile::none(episode_id));
            change(file);
            Ok(())
        }
    }

    struct NoSegments;

    impl SponsorblockRepository for NoSegments {
        fn replace_segments(&self, _: &str, _: &[SponsorSegment]) -> Result<(), CustomError> {
            Ok(())
        }
        fn segments(&self, _: &str) -> Result<Vec<SponsorSegment>, CustomError> {
            Ok(vec![])
        }
        fn preferences(&self, _: UserId) -> Result<Option<SkipPreferences>, CustomError> {
            Ok(None)
        }
        fn save_preferences(&self, _: UserId, _: &SkipPreferences) -> Result<(), CustomError> {
            Ok(())
        }
        fn delete_preferences(&self, _: UserId) -> Result<(), CustomError> {
            Ok(())
        }
    }

    struct Off;

    impl SponsorblockSwitch for Off {
        fn enabled(&self) -> Result<bool, CustomError> {
            Ok(false)
        }
    }

    struct Fixture {
        downloader: Downloader,
        catalog: Arc<FakeCatalog>,
        listener: Arc<RecordingListener>,
        files: Arc<EpisodeFileService>,
    }

    fn settings() -> MediaSettings {
        MediaSettings {
            naming: NamingRules::default(),
            cover_filename: "image".to_string(),
            numbering: true,
            ..MediaSettings::default()
        }
    }

    fn fixture(settings: MediaSettings) -> Fixture {
        let catalog = Arc::new(FakeCatalog::default());
        let media_settings: Arc<dyn MediaSettingsSource> = Arc::new(FixedSettings(settings));
        let listener = Arc::new(RecordingListener::default());
        let files = Arc::new(EpisodeFileService::new(Arc::new(MemoryFiles::default())));
        let nfo = Arc::new(NfoWriter::new(
            catalog.clone(),
            media_settings.clone(),
            FileHandlerType::Local,
        ));
        let sponsorblock = Arc::new(SponsorblockService::new(
            Arc::new(NoSegments),
            SponsorblockClient::with_base_url(reqwest::Client::new(), "http://unused"),
            Arc::new(Off),
        ));
        let mut environment = EnvironmentService::new();
        environment.default_file_handler = FileHandlerType::Local;
        environment.proxy_url = None;
        let downloader = Downloader::new(
            Arc::new(environment),
            catalog.clone(),
            media_settings,
            files.clone(),
            sponsorblock,
            nfo,
            listener.clone(),
            Arc::new(FixedDuration(1594.4)),
        );
        Fixture {
            downloader,
            catalog,
            listener,
            files,
        }
    }

    /// An MP3 with an empty ID3 tag: enough to detect and tag.
    fn mp3_bytes() -> Vec<u8> {
        let dir = std::env::temp_dir().join(format!("podfetch-dl-src-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("source.mp3");
        std::fs::write(&path, [0xFF, 0xFB, 0x90, 0x00]).unwrap();
        id3::Tag::new()
            .write_to_path(&path, id3::Version::Id3v24)
            .unwrap();
        let bytes = std::fs::read(&path).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        bytes
    }

    /// Serves the audio and a cover on its own thread (the download uses
    /// blocking HTTP); returns the base URL.
    fn serve(audio: Vec<u8>) -> String {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            runtime.block_on(async move {
                let app = axum::Router::new()
                    .route(
                        "/episode.mp3",
                        axum::routing::get(move || {
                            let audio = audio.clone();
                            async move { audio }
                        }),
                    )
                    .route(
                        "/cover.jpg",
                        axum::routing::get(|| async { b"jpeg bytes".to_vec() }),
                    );
                let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
                tx.send(listener.local_addr().unwrap()).unwrap();
                let _ = axum::serve(listener, app).await;
            });
        });
        format!("http://{}", rx.recv().unwrap())
    }

    fn podcast(directory: &Path) -> MediaPodcast {
        MediaPodcast {
            id: "p1".to_string(),
            directory_id: "feed-1".to_string(),
            name: "Podcast".to_string(),
            directory_name: directory.to_str().unwrap().to_string(),
            author: Some("Author".to_string()),
            download_location: Some("Local".to_string()),
            ..MediaPodcast::default()
        }
    }

    fn episode(base_url: &str) -> MediaEpisode {
        MediaEpisode {
            id: "e1".to_string(),
            episode_id: "feed-e1".to_string(),
            podcast_id: "p1".to_string(),
            name: "Episode".to_string(),
            url: format!("{base_url}/episode.mp3"),
            image_url: format!("{base_url}/cover.jpg"),
            date_of_recording: "2025-01-02T10:30:00+01:00".to_string(),
            description: "About".to_string(),
            total_time: 1397,
            ..MediaEpisode::default()
        }
    }

    fn podcast_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("podfetch-dl-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_download_stores_tags_and_records_the_episode() {
        let base_url = serve(mp3_bytes());
        let dir = podcast_dir();
        let f = fixture(settings());

        f.downloader
            .download(&episode(&base_url), &podcast(&dir))
            .unwrap();

        let audio = dir.join("Episode/podcast.mp3");
        let image = dir.join("Episode/image.jpg");
        assert!(audio.exists(), "audio stored in the episode directory");
        assert_eq!(std::fs::read(&image).unwrap(), b"jpeg bytes");

        let file = f.files.file("e1").unwrap();
        assert_eq!(file.file_episode_path.as_deref(), audio.to_str());
        assert_eq!(file.file_image_path.as_deref(), image.to_str());

        let tag = id3::Tag::read_from_path(&audio).unwrap();
        use id3::TagLike;
        assert_eq!(tag.title(), Some("3 - Episode"));
        assert_eq!(tag.artist(), Some("Author"));
        assert_eq!(tag.track(), Some(3));
        assert_eq!(*f.catalog.numbered.lock().unwrap(), [true]);
        assert_eq!(*f.catalog.durations.lock().unwrap(), [1594]);

        let told = f.listener.0.lock().unwrap().clone();
        assert_eq!(told, [(audio.to_str().unwrap().to_string(), true)]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn one_cover_for_all_episodes_skips_the_episode_image() {
        let base_url = serve(mp3_bytes());
        let dir = podcast_dir();
        let f = fixture(MediaSettings {
            use_one_cover: true,
            direct_paths: true,
            ..settings()
        });

        f.downloader
            .download(&episode(&base_url), &podcast(&dir))
            .unwrap();

        assert!(dir.join("Episode.mp3").exists(), "direct path");
        assert!(!dir.join("Episode.jpg").exists());
        assert_eq!(f.files.file("e1").unwrap().file_image_path, None);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_failed_download_records_nothing() {
        let dir = podcast_dir();
        let f = fixture(settings());
        let unreachable = episode("http://127.0.0.1:9");

        assert!(f.downloader.download(&unreachable, &podcast(&dir)).is_err());
        assert!(f.files.file("e1").unwrap().is_empty());
        assert!(f.listener.0.lock().unwrap().is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    // Regression: when auto_transcode_opus deletes the source mp3 before
    // tagging runs, tagging used to `.unwrap()` the missing-file error,
    // panicking the download thread. That panic skipped recording the
    // download, so the next polling cycle re-downloaded and re-transcoded
    // the episode, looping forever (issue #2074).
    #[test]
    fn tagging_a_missing_file_is_an_error_not_a_panic() {
        let f = fixture(settings());
        let tmp = std::env::temp_dir().join(format!(
            "podfetch-handle-metadata-missing-{}",
            std::process::id()
        ));
        let paths = FilenameBuilderReturn::new(
            tmp.join("does-not-exist.mp3")
                .to_string_lossy()
                .into_owned(),
            tmp.join("does-not-exist.jpg")
                .to_string_lossy()
                .into_owned(),
        );

        let result = f.downloader.tag_file(
            &paths,
            &MediaEpisode::default(),
            &MediaPodcast::default(),
            &settings(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn existing_file_names_are_used_when_asked() {
        let f = fixture(MediaSettings {
            use_existing_filename: true,
            ..settings()
        });
        let stem = f
            .downloader
            .episode_stem(
                &episode("https://example.com/feed"),
                &f.downloader.settings.for_podcast("p1").unwrap(),
            )
            .unwrap();
        assert_eq!(stem, "episode");
    }
}
