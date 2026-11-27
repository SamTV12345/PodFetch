use crate::services::file::service::{FileService, prepare_podcast_episode_title_to_directory};
use crate::services::podcast_episode_chapter::service::PodcastEpisodeChapterService;
use crate::services::podcast_settings::service::PodcastSettingsService;
use crate::services::settings::service::SettingsService;
use podfetch_media::audio::{Chapter, EpisodeTags, TagFormat};
use podfetch_persistence::podcast::PodcastEntity as Podcast;
use podfetch_persistence::podcast_episode::PodcastEpisodeEntity as PodcastEpisode;

use crate::usecases::podcast_episode::PodcastEpisodeUseCase as PodcastEpisodeService;
use common_infrastructure::config::FileHandlerType;
use common_infrastructure::error::{
    CustomError, CustomErrorInner, ErrorSeverity, map_reqwest_error,
};
use common_infrastructure::http::COMMON_USER_AGENT;
use common_infrastructure::http::{get_async_sync_client, get_sync_client};
use common_infrastructure::runtime::{DEFAULT_IMAGE_URL, ENVIRONMENT_SERVICE, PODCAST_FILENAME};
use podfetch_persistence::db::get_connection;
use podfetch_storage::FileHandleWrapper;
use podfetch_storage::FileRequest;
use podfetch_storage::{
    DetermineFileExtensionReturn, FileType, FilenameBuilder, FilenameBuilderReturn,
    determine_file_extension,
};
use reqwest::header::{ACCEPT_ENCODING, HeaderMap, HeaderValue, USER_AGENT};
use std::path::Path;
use std::thread::sleep;
use std::time::Duration as StdDuration;

pub struct DownloadService {}
const DEFAULT_FALLBACK_IMAGE_BYTES: &[u8] = include_bytes!("../../../../../ui/public/default.jpg");

/// Parse a stored podfetch id (entity rows carry the canonical UUID as a
/// `String`) into a `Uuid`.
fn parse_id(id: &str) -> Result<uuid::Uuid, CustomError> {
    uuid::Uuid::parse_str(id).map_err(|_| CustomErrorInner::NotFound(ErrorSeverity::Warning).into())
}

impl DownloadService {
    pub fn is_default_fallback_image_url(url: &str) -> bool {
        let trimmed = url.trim();
        if trimmed.is_empty() {
            return true;
        }

        trimmed == DEFAULT_IMAGE_URL
            || trimmed == format!("/{DEFAULT_IMAGE_URL}")
            || trimmed.ends_with(&format!("/{DEFAULT_IMAGE_URL}"))
    }

    pub fn get_default_fallback_image_data() -> (String, Vec<u8>) {
        ("jpg".to_string(), DEFAULT_FALLBACK_IMAGE_BYTES.to_vec())
    }

    fn build_binary_sync_client() -> Result<reqwest::blocking::Client, CustomError> {
        get_sync_client(&ENVIRONMENT_SERVICE)
            .no_gzip()
            .no_brotli()
            .no_deflate()
            .no_zstd()
            .build()
            .map_err(map_reqwest_error)
    }

    async fn build_binary_async_client() -> Result<reqwest::Client, CustomError> {
        get_async_sync_client(&ENVIRONMENT_SERVICE)
            .no_gzip()
            .no_brotli()
            .no_deflate()
            .no_zstd()
            .build()
            .map_err(map_reqwest_error)
    }

    fn fetch_bytes_with_retries(url: &str) -> Result<Vec<u8>, CustomError> {
        const MAX_RETRIES: usize = 3;
        for attempt in 1..=MAX_RETRIES {
            let result = Self::build_binary_sync_client()?
                .get(url)
                .header(ACCEPT_ENCODING, "identity")
                .send()
                .map_err(map_reqwest_error)
                .and_then(|resp| resp.bytes().map_err(map_reqwest_error))
                .map(|bytes| bytes.as_ref().to_vec());

            match result {
                Ok(bytes) => return Ok(bytes),
                Err(err) if attempt == MAX_RETRIES => return Err(err),
                Err(err) => {
                    tracing::warn!(
                        "Download attempt {}/{} failed for {}: {}",
                        attempt,
                        MAX_RETRIES,
                        url,
                        err
                    );
                    sleep(StdDuration::from_millis(250 * attempt as u64));
                }
            }
        }
        Err(CustomErrorInner::Conflict(
            "Download failed after retries".to_string(),
            ErrorSeverity::Error,
        )
        .into())
    }

    async fn fetch_bytes_with_retries_async(url: &str) -> Result<Vec<u8>, CustomError> {
        const MAX_RETRIES: usize = 3;
        for attempt in 1..=MAX_RETRIES {
            let client = Self::build_binary_async_client().await?;
            let result = match client
                .get(url)
                .header(ACCEPT_ENCODING, "identity")
                .send()
                .await
            {
                Ok(resp) => resp
                    .bytes()
                    .await
                    .map(|bytes| bytes.as_ref().to_vec())
                    .map_err(map_reqwest_error),
                Err(err) => Err(map_reqwest_error(err)),
            };

            match result {
                Ok(bytes) => return Ok(bytes),
                Err(err) if attempt == MAX_RETRIES => return Err(err),
                Err(err) => {
                    tracing::warn!(
                        "Async download attempt {}/{} failed for {}: {}",
                        attempt,
                        MAX_RETRIES,
                        url,
                        err
                    );
                    tokio::time::sleep(StdDuration::from_millis(250 * attempt as u64)).await;
                }
            }
        }
        Err(CustomErrorInner::Conflict(
            "Download failed after retries".to_string(),
            ErrorSeverity::Error,
        )
        .into())
    }

    pub fn handle_suffix_response(
        dt: DetermineFileExtensionReturn,
        podcast_episode_url: &str,
    ) -> Result<(String, Vec<u8>), CustomError> {
        match dt {
            DetermineFileExtensionReturn::FileExtension(suffix, bytes) => Ok((suffix, bytes)),
            DetermineFileExtensionReturn::String(suffix) => {
                let resp = Self::fetch_bytes_with_retries(podcast_episode_url)?;
                Ok((suffix, resp))
            }
        }
    }

    pub async fn handle_suffix_response_async(
        dt: DetermineFileExtensionReturn,
        podcast_episode_url: &str,
    ) -> Result<(String, Vec<u8>), CustomError> {
        match dt {
            DetermineFileExtensionReturn::FileExtension(suffix, bytes) => Ok((suffix, bytes)),
            DetermineFileExtensionReturn::String(suffix) => {
                let resp = Self::fetch_bytes_with_retries_async(podcast_episode_url).await?;
                Ok((suffix, resp))
            }
        }
    }

    #[tracing::instrument(skip_all, fields(
        episode_id = podcast_episode.id,
        episode_name = %podcast_episode.name,
        podcast_id = podcast.id,
    ))]
    pub fn download_podcast_episode(
        podcast_episode: PodcastEpisode,
        podcast: &Podcast,
    ) -> Result<(), CustomError> {
        let mut header_map = HeaderMap::new();
        header_map.insert(USER_AGENT, HeaderValue::from_static(COMMON_USER_AGENT));
        header_map.insert(ACCEPT_ENCODING, HeaderValue::from_static("identity"));
        let client = get_sync_client(&ENVIRONMENT_SERVICE)
            .default_headers(header_map)
            .no_gzip()
            .no_brotli()
            .no_deflate()
            .timeout(None)
            .no_zstd()
            .build()
            .map_err(map_reqwest_error)?;
        let conn = &mut get_connection();
        let mut podcast_data = Self::handle_suffix_response(
            determine_file_extension(&podcast_episode.url, &client, FileType::Audio),
            &podcast_episode.url,
        )?;
        if podfetch_media::audio::convert::is_hls_playlist(&podcast_data.1) {
            tracing::info!(
                "Episode {} is served as an HLS playlist, downloading via ffmpeg",
                podcast_episode.episode_id
            );
            podcast_data =
                podfetch_media::audio::convert::download_hls_episode(&podcast_episode.url)?;
        }
        let settings_in_db = SettingsService::shared()
            .get_settings()?
            .ok_or_else(|| CustomError::from(CustomErrorInner::Unknown(ErrorSeverity::Error)))?;
        let podcast_settings_override =
            PodcastSettingsService::get_settings_for_podcast(parse_id(&podcast.id)?)?;
        let use_one_cover_for_all_episodes = podfetch_media::rules::use_one_cover_from(
            podcast_settings_override
                .as_ref()
                .map(|s| (s.activated, s.use_one_cover_for_all_episodes)),
            settings_in_db.use_one_cover_for_all_episodes,
        );
        let episode_has_own_image =
            !Self::is_default_fallback_image_url(&podcast_episode.image_url);
        let podcast_main_image_downloaded = FileService::check_if_podcast_main_image_downloaded(
            &podcast.clone().directory_id,
            conn,
        );

        // Download when: episode has its own image, or the podcast main image
        // hasn't been downloaded yet (and we're not forced to use one cover).
        let should_download_main_image = podfetch_media::rules::should_download_image(
            use_one_cover_for_all_episodes,
            episode_has_own_image,
            podcast_main_image_downloaded,
        );

        let mut image_data = if should_download_main_image {
            let image_url = if episode_has_own_image {
                Some(podcast_episode.image_url.as_str())
            } else if !Self::is_default_fallback_image_url(&podcast.original_image_url) {
                tracing::info!(
                    "Episode {} has no image, falling back to podcast image",
                    podcast_episode.episode_id
                );
                Some(podcast.original_image_url.as_str())
            } else {
                None
            };

            match image_url {
                Some(url) => Some(Self::handle_suffix_response(
                    determine_file_extension(url, &client, FileType::Image),
                    url,
                )?),
                None => Some(Self::get_default_fallback_image_data()),
            }
        } else {
            None
        };

        let episode_stem = prepare_podcast_episode_title_to_directory(podcast_episode.clone())?;
        let cover_filename =
            crate::services::nfo::service::resolve_cover_filename(parse_id(&podcast.id)?);
        let paths = FilenameBuilder::default()
            .with_podcast_directory(&podcast.directory_name)
            .with_episode_stem(&episode_stem)
            .with_suffix(&podcast_data.0)
            .with_image_suffix(
                &image_data
                    .as_ref()
                    .map_or_else(|| "jpg".to_string(), |data| data.0.clone()),
            )
            .with_image_filename(&cover_filename)
            .with_filename(PODCAST_FILENAME)
            .with_direct_paths(settings_in_db.direct_paths)
            .build(|directory| {
                FileService::ensure_podcast_episode_directory_available(&directory, podcast)
            })?;

        if !FileHandleWrapper::path_exists(
            &podcast.directory_name,
            FileRequest::Directory,
            &ENVIRONMENT_SERVICE.default_file_handler,
        ) {
            FileHandleWrapper::create_dir(
                &podcast.directory_name,
                &ENVIRONMENT_SERVICE.default_file_handler,
            )?;
        }

        // `paths.filename` is a `String`, so its parent is valid UTF-8 too.
        if let Some(parent) = Path::new(&paths.filename).parent().and_then(Path::to_str)
            && !FileHandleWrapper::path_exists(
                parent,
                FileRequest::Directory,
                &ENVIRONMENT_SERVICE.default_file_handler,
            )
        {
            FileHandleWrapper::create_dir(parent, &ENVIRONMENT_SERVICE.default_file_handler)?;
        }

        let image_written =
            should_download_main_image && image_data.as_ref().is_some_and(|d| !d.1.is_empty());
        if image_written && let Some(ref mut data) = image_data {
            FileHandleWrapper::write_file(
                &paths.image_filename,
                data.1.as_mut_slice(),
                &ENVIRONMENT_SERVICE.default_file_handler,
            )?;
        }

        FileHandleWrapper::write_file(
            &paths.filename,
            podcast_data.1.as_mut_slice(),
            &ENVIRONMENT_SERVICE.default_file_handler,
        )?;

        // Read chapters and embed tags BEFORE transcoding. Opus has no
        // id3/mp4-atom metadata, and transcoding would also delete the
        // source file we need to read chapters from. Doing this first means
        // chapters are preserved in the DB even when the source is removed
        // by the transcode step below.
        let result = Self::handle_metadata_insertion(&paths, &podcast_episode, podcast);
        if let Ok(chapters) = &result {
            tracing::info!("Inserting chapters for episode {}", podcast_episode.id);
            for chapter in chapters {
                let res = PodcastEpisodeChapterService::default_service()
                    .save_chapter(chapter, &podcast_episode);
                if let Err(err) = res {
                    tracing::error!(
                        "Error while saving chapter for episode {}: {}",
                        podcast_episode.id,
                        err
                    );
                }
            }
        }

        if let Err(err) = result {
            tracing::error!("Error handling metadata insertion: {err:?}");
        }

        // Transcripts: download + parse feed transcripts.
        // Non-fatal — must never fail the download.
        let transcript_service =
            crate::services::transcript::service::TranscriptService::default_service();
        if let Err(err) =
            transcript_service.process_pending_after_download(&podcast_episode, &paths.filename)
        {
            tracing::error!(
                "Error processing transcripts for episode {}: {err}",
                podcast_episode.id
            );
        }

        // Auto-transcribe: enqueue a generation job when this podcast opted in
        // and the episode doesn't already have a usable (feed) transcript in
        // flight. Non-fatal — must never fail the download.
        let auto_transcribe = podcast_settings_override
            .as_ref()
            .is_some_and(|settings| settings.auto_transcribe);
        if ENVIRONMENT_SERVICE.transcription_config.is_some()
            && auto_transcribe
            && let Ok(episode_uuid) = parse_id(&podcast_episode.id)
            && transcript_service
                .needs_generated_transcript(episode_uuid)
                .unwrap_or(false)
            && let Err(err) = transcript_service.enqueue_job(episode_uuid)
        {
            tracing::error!("Error enqueuing transcription job: {err}");
        }

        // SponsorBlock: fetch + store segments for YouTube-sourced episodes.
        // Non-fatal — must never fail the download.
        match crate::media::fetch_sponsor_segments_blocking(&podcast_episode) {
            Ok(n) if n > 0 => tracing::info!(
                "Stored {n} SponsorBlock segments for episode {}",
                podcast_episode.id
            ),
            Ok(_) => {}
            Err(err) => tracing::warn!(
                "SponsorBlock fetch failed for episode {}: {err}",
                podcast_episode.id
            ),
        }

        // The feed's itunes:duration can be shorter than the file (dynamic ad
        // insertion), so prefer the real duration. Non-fatal — must never fail
        // the download.
        Self::sync_total_time_with_file(&podcast_episode, &paths.filename);

        let final_episode_path = if settings_in_db.auto_transcode_opus {
            match podfetch_media::audio::convert::transcode_to_opus(&paths.filename) {
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
        crate::services::nfo::service::regenerate_for_episode(
            podcast,
            &podcast_episode,
            &final_episode_path,
        );

        PodcastEpisodeService::update_local_paths(
            &podcast_episode.id,
            if image_written {
                &paths.image_filename
            } else {
                ""
            },
            &final_episode_path,
        )?;
        Ok(())
    }

    fn sync_total_time_with_file(podcast_episode: &PodcastEpisode, file_path: &str) {
        let probed = match podfetch_audiobooks::services::audio_probe::probe_audio_file(
            std::path::Path::new(file_path),
        ) {
            Ok(probed) => probed,
            Err(err) => {
                tracing::debug!(
                    "Could not probe duration of episode {}: {err}",
                    podcast_episode.id
                );
                return;
            }
        };
        if let Some(total_time) =
            podfetch_media::rules::duration_to_store(podcast_episode.total_time, probed.duration)
            && let Err(err) =
                PodcastEpisodeService::update_total_time(&podcast_episode.episode_id, total_time)
        {
            tracing::warn!(
                "Could not update duration of episode {}: {err}",
                podcast_episode.id
            );
        }
    }

    /// Tags the downloaded file with the episode's metadata and returns the
    /// chapters embedded in it. Files on S3 are left alone. A failure to
    /// tag is logged; only an unreadable or unsupported file is an error.
    pub fn handle_metadata_insertion(
        paths: &FilenameBuilderReturn,
        podcast_episode: &PodcastEpisode,
        podcast: &Podcast,
    ) -> Result<Vec<Chapter>, CustomError> {
        if ENVIRONMENT_SERVICE.default_file_handler == FileHandlerType::S3 {
            return Ok(vec![]);
        }

        let format = TagFormat::of_file(&paths.filename)?;
        let chapters = format.read_chapters(&paths.filename)?;
        let written = Self::episode_tags(podcast_episode, podcast).and_then(|tags| {
            format.write(
                &paths.filename,
                &paths.image_filename,
                &tags,
                &mut |numbered| {
                    PodcastEpisodeService::update_episode_numbering_processed(
                        numbered,
                        &podcast_episode.episode_id,
                    )
                },
            )
        });
        if let Err(err) = written {
            tracing::error!("Error updating metadata: {err:?}");
        }
        Ok(chapters)
    }

    /// What goes into the file's tags, translated from the catalog.
    fn episode_tags(
        podcast_episode: &PodcastEpisode,
        podcast: &Podcast,
    ) -> Result<EpisodeTags, CustomError> {
        let index = PodcastEpisodeService::get_position_of_episode(
            &podcast_episode.date_of_recording,
            parse_id(&podcast_episode.podcast_id)?,
        )?;
        let numbering_enabled = Self::episode_numbering_enabled(parse_id(&podcast.id)?)?;
        let track_number = PodcastEpisodeService::get_track_number_for_episode(
            parse_id(&podcast.id)?,
            &podcast_episode.date_of_recording,
        )
        .inspect_err(|e| tracing::error!("Error getting track number: {e:?}"))
        .ok();
        Ok(EpisodeTags {
            title: podfetch_media::rules::resolve_episode_title(
                &podcast_episode.name,
                podcast_episode.episode_numbering_processed,
                numbering_enabled,
                index,
            ),
            artist: podcast.author.clone(),
            album: podcast.name.clone(),
            genre: podcast.keywords.clone(),
            language: podcast.language.clone(),
            comment: podcast_episode.description.clone(),
            date_of_recording: podcast_episode.date_of_recording.clone(),
            track_number,
        })
    }

    /// Resolve whether episode-number title prefixing applies to a podcast.
    ///
    /// The per-podcast `episode_numbering` flag is OR-ed with the instance-wide
    /// default from the global settings, so enabling numbering in general
    /// settings turns it on for every podcast unless it is already on per
    /// podcast. This mirrors how `auto_download` layers the per-podcast value
    /// over the global default.
    fn episode_numbering_enabled(podcast_id: uuid::Uuid) -> Result<bool, CustomError> {
        let per_podcast = PodcastSettingsService::get_settings_for_podcast(podcast_id)?
            .map(|s| s.episode_numbering);
        let global = SettingsService::shared()
            .get_settings()?
            .map(|s| s.episode_numbering);
        Ok(podfetch_media::rules::numbering_from(per_podcast, global))
    }
}

#[cfg(test)]
mod tests {

    use super::{DownloadService, FilenameBuilderReturn, Podcast, PodcastEpisode};

    #[test]
    fn empty_stored_value_is_treated_as_default_image() {
        assert!(DownloadService::is_default_fallback_image_url(""));
    }

    #[test]
    fn whitespace_only_is_treated_as_default_image() {
        assert!(DownloadService::is_default_fallback_image_url("   "));
    }

    fn make_podcast() -> Podcast {
        Podcast {
            id: uuid::Uuid::nil().to_string(),
            legacy_id: None,
            name: "p".into(),
            directory_id: "1".into(),
            rssfeed: "https://x".into(),
            image_url: "https://x/img.jpg".into(),
            summary: None,
            language: None,
            explicit: None,
            keywords: None,
            last_build_date: None,
            author: None,
            active: true,
            original_image_url: "https://x/img".into(),
            directory_name: "podcasts/p".into(),
            download_location: Some("Local".into()),
            guid: None,
            added_by: None,
        }
    }

    // Regression: when auto_transcode_opus deletes the source mp3 before
    // handle_metadata_insertion runs, the function used to `.unwrap()` the
    // missing-file error from `FileFormat::from_file`, panicking the download
    // thread. That panic skipped `update_podcast_episode_status`, leaving
    // `download_location` NULL — so the next polling cycle re-downloaded and
    // re-transcoded the episode, looping forever (issue #2074).
    #[test]
    fn handle_metadata_insertion_returns_err_when_audio_file_is_missing() {
        let tmp = std::env::temp_dir().join(format!(
            "podfetch-handle-metadata-missing-{}",
            std::process::id()
        ));
        let missing_audio = tmp.join("does-not-exist.mp3");
        let missing_image = tmp.join("does-not-exist.jpg");
        let paths = FilenameBuilderReturn::new(
            missing_audio.to_string_lossy().into_owned(),
            missing_image.to_string_lossy().into_owned(),
        );
        let episode = PodcastEpisode::default();
        let podcast = make_podcast();

        let result = DownloadService::handle_metadata_insertion(&paths, &episode, &podcast);

        assert!(
            result.is_err(),
            "handle_metadata_insertion must return Err when the audio file is missing, \
             not panic — otherwise the download thread dies and the episode is \
             re-downloaded/re-transcoded on every poll",
        );
    }

    // ── check_if_podcast_main_image_downloaded integration tests ─────────

    use crate::services::file::service::FileService;
    use podfetch_domain::podcast::NewPodcast;
    use podfetch_domain::podcast::PodcastRepository;
    use podfetch_persistence::db::{self, database};
    use podfetch_persistence::podcast::DieselPodcastRepository;
    use std::io::Write;

    fn seed_podcast(image_url: &str, directory_id: &str) -> String {
        crate::test_support::tests::ensure_test_env_vars();
        let repo = DieselPodcastRepository::new(database());
        let id = uuid::Uuid::new_v4().to_string();
        let podcast = NewPodcast {
            name: "test-podcast".into(),
            directory_id: directory_id.to_string(),
            rssfeed: "https://example.com/feed.xml".into(),
            image_url: image_url.to_string(),
            directory_name: format!("podcasts/{directory_id}"),
            added_by: None,
        };
        repo.create(podcast).expect("seed podcast");
        id
    }

    fn cleanup_podcast(directory_id: &str) {
        use diesel::RunQueryDsl;
        use podfetch_persistence::db::get_connection;
        let _ = diesel::sql_query(format!(
            "DELETE FROM podcasts WHERE directory_id = '{directory_id}'"
        ))
        .execute(&mut get_connection());
    }

    #[test]
    #[serial_test::serial]
    fn main_image_not_downloaded_when_podcast_not_found() {
        crate::test_support::tests::ensure_test_env_vars();
        let mut conn = db::get_connection();
        assert!(
            !FileService::check_if_podcast_main_image_downloaded("nonexistent-id", &mut conn),
            "non-existent podcast → image not downloaded"
        );
    }

    #[test]
    #[serial_test::serial]
    fn main_image_not_downloaded_when_url_is_http() {
        let dir_id = format!("test-http-{}", uuid::Uuid::new_v4().simple());
        seed_podcast("https://example.com/cover.jpg", &dir_id);

        crate::test_support::tests::ensure_test_env_vars();
        let mut conn = db::get_connection();
        let result = FileService::check_if_podcast_main_image_downloaded(&dir_id, &mut conn);
        cleanup_podcast(&dir_id);
        assert!(!result, "HTTP image_url → not downloaded");
    }

    #[test]
    #[serial_test::serial]
    fn main_image_not_downloaded_when_local_file_missing() {
        let dir_id = format!("test-local-missing-{}", uuid::Uuid::new_v4().simple());
        seed_podcast("/tmp/does-not-exist-928347.jpg", &dir_id);

        crate::test_support::tests::ensure_test_env_vars();
        let mut conn = db::get_connection();
        let result = FileService::check_if_podcast_main_image_downloaded(&dir_id, &mut conn);
        cleanup_podcast(&dir_id);
        assert!(!result, "local path but file missing → not downloaded");
    }

    #[test]
    #[serial_test::serial]
    fn main_image_downloaded_when_local_file_exists() {
        let dir_id = format!("test-local-exists-{}", uuid::Uuid::new_v4().simple());
        let tmp = std::env::temp_dir().join(format!(
            "podfetch-test-cover-{}.jpg",
            uuid::Uuid::new_v4().simple()
        ));
        {
            let mut f = std::fs::File::create(&tmp).expect("create temp file");
            f.write_all(b"fake-image-data").expect("write");
        }
        seed_podcast(&tmp.to_string_lossy(), &dir_id);

        crate::test_support::tests::ensure_test_env_vars();
        let mut conn = db::get_connection();
        let result = FileService::check_if_podcast_main_image_downloaded(&dir_id, &mut conn);
        cleanup_podcast(&dir_id);
        let _ = std::fs::remove_file(&tmp);
        assert!(result, "local file exists → downloaded");
    }
}
