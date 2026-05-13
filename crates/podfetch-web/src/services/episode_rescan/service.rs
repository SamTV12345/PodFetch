use crate::services::download::service::DownloadService;
use crate::services::file::service::{FileService, prepare_podcast_episode_title_to_directory};
use crate::services::podcast::service::PodcastService;
use crate::services::settings::service::SettingsService;
use crate::usecases::podcast_episode::PodcastEpisodeUseCase as PodcastEpisodeService;
use common_infrastructure::config::FileHandlerType;
use common_infrastructure::error::{CustomError, CustomErrorInner, ErrorSeverity};
use common_infrastructure::runtime::{PODCAST_FILENAME, PODCAST_IMAGENAME};
use podfetch_persistence::podcast::PodcastEntity as Podcast;
use podfetch_persistence::podcast_episode::PodcastEpisodeEntity as PodcastEpisode;
use podfetch_storage::{FileHandleWrapper, FileRequest, FilenameBuilder, FilenameBuilderReturn};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Toggles for re-applying the current settings to already-downloaded
/// episodes. Each field is opt-in so an empty body keeps the historical
/// "chapter rescan only" behaviour of `/settings/rescan-episodes`.
#[derive(Debug, Default, Clone, Copy, Deserialize, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct RescanOptions {
    /// Rename / move the audio + image file to the path the current naming
    /// settings would produce.
    pub apply_filenames: bool,
    /// If `auto_transcode_opus` is on and the file is still mp3, convert it
    /// to opus (same ffmpeg call the downloader uses).
    pub apply_transcode: bool,
    /// If `use_one_cover_for_all_episodes` is on, remove the per-episode
    /// cover image from disk.
    pub apply_covers: bool,
    /// Rewrite embedded ID3 / MP4 tags (title, artist, album, track, cover,
    /// episode-number prefix).
    pub apply_metadata: bool,
}

impl RescanOptions {
    pub fn any_enabled(&self) -> bool {
        self.apply_filenames || self.apply_transcode || self.apply_covers || self.apply_metadata
    }
}

#[derive(Debug, Default, Clone)]
pub struct RescanApplyStats {
    pub renamed: usize,
    pub transcoded: usize,
    pub covers_consolidated: usize,
    pub metadata_refreshed: usize,
    pub errors: usize,
}

pub struct EpisodeRescanService;

impl EpisodeRescanService {
    /// Re-apply the current settings to a single downloaded episode. Steps
    /// run in the order
    ///   transcode -> rename/move -> metadata -> cover consolidation
    /// so each later step sees the final on-disk file produced by the
    /// previous one. Each step is opt-in via `opts`.
    ///
    /// Episodes without a `file_episode_path`, episodes whose download
    /// location is S3, and missing files are skipped silently — the caller
    /// just won't see them in the per-step counters.
    pub fn apply_to_episode(
        episode: &PodcastEpisode,
        opts: &RescanOptions,
        stats: &mut RescanApplyStats,
    ) -> Result<(), CustomError> {
        let Some(current_audio_path) = episode.file_episode_path.clone() else {
            return Ok(());
        };

        let download_location =
            FileHandlerType::from(episode.download_location.as_deref().unwrap_or("Local"));
        if download_location == FileHandlerType::S3 {
            // The transcode + rename + tag-write helpers all assume local
            // filesystem; the existing downloader short-circuits on S3 too.
            return Ok(());
        }

        if !FileHandleWrapper::path_exists(
            &current_audio_path,
            FileRequest::File,
            &download_location,
        ) {
            tracing::warn!(
                "Episode {} has file_episode_path {} but the file is missing; skipping",
                episode.episode_id,
                current_audio_path
            );
            // Even though we can't operate on the file, its old per-episode
            // directory may be a leftover from a previous (partial) rename
            // pass. If it's truly empty, clean it up so the user doesn't
            // accumulate dead directories.
            if opts.apply_filenames && download_location == FileHandlerType::Local {
                let podcast_root =
                    PodcastService::get_podcast_by_id(episode.podcast_id).directory_name;
                if let Some(parent) = Path::new(&current_audio_path).parent()
                    && let Some(parent_str) = parent.to_str()
                {
                    Self::remove_empty_dir_up_to(parent_str, &podcast_root);
                }
            }
            return Ok(());
        }

        let podcast = PodcastService::get_podcast_by_id(episode.podcast_id);
        let settings = SettingsService::shared()
            .get_settings()?
            .ok_or_else(|| -> CustomError {
                CustomErrorInner::Conflict(
                    "settings row missing".to_string(),
                    ErrorSeverity::Error,
                )
                .into()
            })?;

        let mut working_audio_path = current_audio_path.clone();

        // Step 1: transcode mp3 -> opus if requested and applicable.
        if opts.apply_transcode
            && settings.auto_transcode_opus
            && extension_lower(&working_audio_path).as_deref() == Some("mp3")
        {
            match DownloadService::transcode_to_opus(&working_audio_path) {
                Ok(new_path) => {
                    working_audio_path = new_path;
                    stats.transcoded += 1;
                }
                Err(err) => {
                    tracing::warn!(
                        "Transcode to opus failed for episode {}: {err}",
                        episode.episode_id
                    );
                    stats.errors += 1;
                }
            }
        }

        let working_image_path = episode.file_image_path.clone().unwrap_or_default();

        // Step 2: rename / move to the path the current naming settings
        // would produce. We persist new paths to the DB even when we skip
        // metadata refresh, so the player always points at the live file.
        let mut final_audio_path = working_audio_path.clone();
        let mut final_image_path = working_image_path.clone();

        if opts.apply_filenames {
            let target = Self::compute_target_paths(
                episode,
                &podcast,
                &working_audio_path,
                &working_image_path,
                settings.direct_paths,
            )?;

            // Remember parents we moved files out of so we can clean up
            // empty episode subdirectories at the end of this step.
            let mut vacated_dirs: Vec<String> = Vec::new();
            let remember = |dirs: &mut Vec<String>, path: &str| {
                if let Some(parent) = Path::new(path).parent()
                    && let Some(parent_str) = parent.to_str()
                    && !parent_str.is_empty()
                    && !dirs.iter().any(|d| d == parent_str)
                {
                    dirs.push(parent_str.to_string());
                }
            };

            if target.filename != working_audio_path {
                if let Some(parent) = Path::new(&target.filename).parent()
                    && let Some(parent_str) = parent.to_str()
                    && !FileHandleWrapper::path_exists(
                        parent_str,
                        FileRequest::Directory,
                        &download_location,
                    )
                {
                    FileHandleWrapper::create_dir(parent_str, &download_location)?;
                }

                remember(&mut vacated_dirs, &working_audio_path);
                FileHandleWrapper::rename_file(
                    &working_audio_path,
                    &target.filename,
                    &download_location,
                )?;
                final_audio_path = target.filename.clone();
                stats.renamed += 1;
            }

            if !working_image_path.is_empty()
                && target.image_filename != working_image_path
                && FileHandleWrapper::path_exists(
                    &working_image_path,
                    FileRequest::File,
                    &download_location,
                )
            {
                remember(&mut vacated_dirs, &working_image_path);
                FileHandleWrapper::rename_file(
                    &working_image_path,
                    &target.image_filename,
                    &download_location,
                )?;
                final_image_path = target.image_filename;
            }

            // Clean up any now-empty source directories. We never recurse
            // up past the podcast directory, and we only remove dirs that
            // are genuinely empty — anything else (stray files, chapter
            // artifacts, sub-podcast dirs) is left alone.
            if download_location == FileHandlerType::Local {
                for dir in vacated_dirs {
                    Self::remove_empty_dir_up_to(&dir, &podcast.directory_name);
                }
            }
        }

        // Persist the new paths whenever they changed (either from
        // transcode or rename).
        if final_audio_path != current_audio_path
            || (!final_image_path.is_empty() && Some(&final_image_path) != episode.file_image_path.as_ref())
        {
            PodcastEpisodeService::update_local_paths(
                &episode.episode_id,
                &final_image_path,
                &final_audio_path,
            )?;
        }

        // Step 3: re-write embedded metadata (ID3 / MP4) into the final
        // file. handle_metadata_insertion opens the image at
        // `image_filename` to embed a cover when one isn't already present,
        // so we run this BEFORE cover consolidation.
        if opts.apply_metadata {
            let paths = FilenameBuilderReturn::new(final_audio_path.clone(), final_image_path.clone());
            // Only attempt embedding if the cover file we'd read actually
            // exists; the downloader's helper unwraps on File::open.
            let cover_ok = final_image_path.is_empty()
                || FileHandleWrapper::path_exists(
                    &final_image_path,
                    FileRequest::File,
                    &download_location,
                );
            if cover_ok {
                match DownloadService::handle_metadata_insertion(&paths, episode, &podcast) {
                    Ok(_chapters) => {
                        stats.metadata_refreshed += 1;
                    }
                    Err(err) => {
                        tracing::warn!(
                            "Metadata refresh failed for episode {}: {err}",
                            episode.episode_id
                        );
                        stats.errors += 1;
                    }
                }
            } else {
                tracing::debug!(
                    "Skipping metadata refresh for episode {}: cover file missing",
                    episode.episode_id
                );
            }
        }

        // Step 4: cover consolidation. The episode is downloaded (S3 was
        // already filtered out at the top), so file_image_path points at a
        // local file. Trust the DB and delete it. The only carve-out is
        // when that path happens to also be the shared podcast cover —
        // deleting it would break every other episode that references it.
        //
        // `podcast.image_url` stores the URL-encoded path (last segment
        // percent-encoded by `map_to_local_url`), so we derive the RAW
        // filesystem path of the shared cover from `directory_name` plus
        // the extension. We must NOT write the URL-encoded form into
        // `file_image_path` — the serve layer encodes it again, yielding
        // double-encoded `%2520`.
        let shared_cover_fs_path = shared_cover_fs_path(&podcast);
        if opts.apply_covers && !final_image_path.is_empty() {
            let is_shared_cover = shared_cover_fs_path
                .as_deref()
                .is_some_and(|p| p == final_image_path);

            if is_shared_cover {
                tracing::debug!(
                    "Skipping cover for episode {}: {} is the shared podcast cover",
                    episode.episode_id,
                    final_image_path,
                );
            } else if !FileHandleWrapper::path_exists(
                &final_image_path,
                FileRequest::File,
                &download_location,
            ) {
                tracing::debug!(
                    "Per-episode cover {} for episode {} already missing from disk",
                    final_image_path,
                    episode.episode_id
                );
            } else {
                match FileHandleWrapper::remove_file(&final_image_path, &download_location) {
                    Ok(()) => {
                        stats.covers_consolidated += 1;
                        tracing::info!(
                            "Removed per-episode cover {} for episode {}",
                            final_image_path,
                            episode.episode_id
                        );
                        let new_image_db_value =
                            shared_cover_fs_path.as_deref().unwrap_or("");
                        if let Err(err) = PodcastEpisodeService::update_local_paths(
                            &episode.episode_id,
                            new_image_db_value,
                            &final_audio_path,
                        ) {
                            tracing::warn!(
                                "Could not repoint file_image_path for {}: {err}",
                                episode.episode_id
                            );
                        }
                    }
                    Err(err) => {
                        tracing::warn!(
                            "Could not remove per-episode cover {} for {}: {err}",
                            final_image_path,
                            episode.episode_id
                        );
                        stats.errors += 1;
                    }
                }
            }
        }
        Ok(())
    }

    fn compute_target_paths(
        episode: &PodcastEpisode,
        podcast: &Podcast,
        current_audio_path: &str,
        current_image_path: &str,
        direct_paths: bool,
    ) -> Result<FilenameBuilderReturn, CustomError> {
        let suffix = extension_lower(current_audio_path).unwrap_or_else(|| "mp3".to_string());
        let image_suffix = extension_lower(current_image_path).unwrap_or_else(|| "jpg".to_string());

        let episode_stem = prepare_podcast_episode_title_to_directory(episode.clone())?;

        // When `direct_paths` is off, FilenameBuilder calls into our
        // resolver to find a free `<stem>-<n>` directory. If the episode
        // currently lives in a `<stem>-<n>` directory, prefer keeping that
        // one to avoid pointlessly spinning to `-2`, `-3`, ...
        let preferred_dir = if !direct_paths {
            Path::new(current_audio_path)
                .parent()
                .and_then(|p| p.to_str())
                .map(|s| s.to_string())
        } else {
            None
        };

        let download_location =
            FileHandlerType::from(episode.download_location.as_deref().unwrap_or("Local"));

        FilenameBuilder::default()
            .with_podcast_directory(&podcast.directory_name)
            .with_episode_stem(&episode_stem)
            .with_suffix(&suffix)
            .with_image_suffix(&image_suffix)
            .with_image_filename(PODCAST_IMAGENAME)
            .with_filename(PODCAST_FILENAME)
            .with_direct_paths(direct_paths)
            .build(|directory| -> Result<String, CustomError> {
                if let Some(existing) = preferred_dir.as_ref()
                    && existing.starts_with(&directory)
                    && FileHandleWrapper::path_exists(
                        existing,
                        FileRequest::Directory,
                        &download_location,
                    )
                {
                    return Ok(existing.clone());
                }
                FileService::ensure_podcast_episode_directory_available(&directory, podcast)
            })
    }

    /// Walk up from `start_dir`, removing each directory that is now empty,
    /// stopping before we'd touch `podcast_root`. Used to clean up the
    /// `<stem>-<n>` episode subdirectories left behind after a rename when
    /// `direct_paths` is off.
    fn remove_empty_dir_up_to(start_dir: &str, podcast_root: &str) {
        let normalise = |s: &str| s.replace('\\', "/");
        let root_n = normalise(podcast_root);
        let mut current = Path::new(start_dir).to_path_buf();

        loop {
            let current_n = normalise(&current.to_string_lossy());
            if current_n == root_n || !current_n.starts_with(&root_n) {
                break;
            }
            // Only descendants of podcast_root are eligible — never the
            // podcast directory itself.
            let mut iter = match std::fs::read_dir(&current) {
                Ok(i) => i,
                Err(err) => {
                    tracing::debug!(
                        "Cannot read dir {} while cleaning empty episode dirs: {err}",
                        current.display()
                    );
                    break;
                }
            };
            if iter.next().is_some() {
                break;
            }
            if let Err(err) = std::fs::remove_dir(&current) {
                tracing::debug!(
                    "Could not remove empty dir {}: {err}",
                    current.display()
                );
                break;
            }
            tracing::info!("Removed empty episode directory {}", current.display());
            match current.parent() {
                Some(parent) => current = parent.to_path_buf(),
                None => break,
            }
        }
    }
}

fn extension_lower(path: &str) -> Option<String> {
    Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
}

/// Derive the on-disk path of a podcast's shared cover. `podcast.image_url`
/// is a URL-encoded path (produced by `map_to_local_url`), so we can't use
/// it directly as a filesystem path. The shared cover always lives at
/// `<directory_name>/image.<ext>`; we just need the extension, which we
/// take from the stored image_url (decoded if necessary).
fn shared_cover_fs_path(podcast: &Podcast) -> Option<String> {
    if podcast.directory_name.is_empty() {
        return None;
    }
    let ext = extension_lower(&podcast.image_url)?;
    Some(format!("{}/image.{}", podcast.directory_name, ext))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rescan_options_any_enabled_reflects_individual_toggles() {
        assert!(!RescanOptions::default().any_enabled());
        assert!(
            RescanOptions {
                apply_filenames: true,
                ..RescanOptions::default()
            }
            .any_enabled()
        );
        assert!(
            RescanOptions {
                apply_metadata: true,
                ..RescanOptions::default()
            }
            .any_enabled()
        );
    }

    #[test]
    fn rescan_options_deserialize_from_empty_body_uses_defaults() {
        let parsed: RescanOptions = serde_json::from_str("{}").unwrap();
        assert!(!parsed.any_enabled());
    }

    #[test]
    fn rescan_options_deserialize_camel_case() {
        let parsed: RescanOptions = serde_json::from_str(
            r#"{"applyFilenames": true, "applyTranscode": true}"#,
        )
        .unwrap();
        assert!(parsed.apply_filenames);
        assert!(parsed.apply_transcode);
        assert!(!parsed.apply_covers);
        assert!(!parsed.apply_metadata);
    }

    #[test]
    fn extension_lower_handles_uppercase_and_missing() {
        assert_eq!(extension_lower("/tmp/x.MP3").as_deref(), Some("mp3"));
        assert_eq!(extension_lower("/tmp/no-ext").as_deref(), None);
    }

    fn make_podcast(image_url: &str, directory_name: &str) -> Podcast {
        Podcast {
            id: 1,
            name: "p".into(),
            directory_id: "1".into(),
            rssfeed: "https://x".into(),
            image_url: image_url.into(),
            summary: None,
            language: None,
            explicit: None,
            keywords: None,
            last_build_date: None,
            author: None,
            active: true,
            original_image_url: "https://x/img".into(),
            directory_name: directory_name.into(),
            download_location: Some("Local".into()),
            guid: None,
            added_by: None,
        }
    }

    #[test]
    fn shared_cover_fs_path_uses_directory_name_and_extension_not_url() {
        // image_url is URL-encoded (last segment percent-encoded). We must
        // derive the raw FS path from directory_name, not from image_url
        // itself, so the result is safe to store in file_image_path.
        let podcast = make_podcast(
            "podcasts/1%20Minute%20Podcast/image.jpg",
            "podcasts/1 Minute Podcast",
        );
        assert_eq!(
            shared_cover_fs_path(&podcast).as_deref(),
            Some("podcasts/1 Minute Podcast/image.jpg")
        );
    }

    #[test]
    fn shared_cover_fs_path_returns_none_without_extension() {
        let podcast = make_podcast("", "podcasts/x");
        assert_eq!(shared_cover_fs_path(&podcast), None);
    }

    #[test]
    fn remove_empty_dir_up_to_walks_up_to_podcast_root() {
        let tmp = std::env::temp_dir().join(format!(
            "podfetch-rescan-cleanup-{}",
            std::process::id()
        ));
        let root = tmp.join("podcast-root");
        let level1 = root.join("episode-1");
        let level2 = level1.join("sub");
        std::fs::create_dir_all(&level2).unwrap();

        EpisodeRescanService::remove_empty_dir_up_to(
            level2.to_str().unwrap(),
            root.to_str().unwrap(),
        );

        assert!(!level2.exists());
        assert!(!level1.exists());
        assert!(root.exists(), "podcast root itself must be left alone");

        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn remove_empty_dir_up_to_stops_at_non_empty_parent() {
        let tmp = std::env::temp_dir().join(format!(
            "podfetch-rescan-cleanup-nonempty-{}",
            std::process::id()
        ));
        let root = tmp.join("podcast-root");
        let level1 = root.join("episode-1");
        let level2 = level1.join("sub");
        std::fs::create_dir_all(&level2).unwrap();
        std::fs::write(level1.join("keepme.txt"), b"x").unwrap();

        EpisodeRescanService::remove_empty_dir_up_to(
            level2.to_str().unwrap(),
            root.to_str().unwrap(),
        );

        assert!(!level2.exists(), "empty leaf should be removed");
        assert!(level1.exists(), "non-empty parent must be kept");

        std::fs::remove_dir_all(&tmp).ok();
    }
}
