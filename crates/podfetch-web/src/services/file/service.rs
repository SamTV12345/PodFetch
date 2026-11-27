use podfetch_catalog::persistence::podcast::PodcastEntity as Podcast;

use std::path::Path;

use crate::podcast::PodcastInsertModel;
use crate::podcast_settings::PodcastSetting;
use crate::services::podcast::service::PodcastService;
use crate::services::podcast_settings::service::PodcastSettingsService;
use crate::services::settings::service::SettingsService;
use crate::usecases::podcast_episode::PodcastEpisodeUseCase as PodcastEpisodeService;
use common_infrastructure::config::FileHandlerType;
use common_infrastructure::error::{
    CustomError, CustomErrorInner, ErrorSeverity, map_reqwest_error,
};
use common_infrastructure::http::get_http_client;
use common_infrastructure::rss::RSSFeedParser;
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
use podfetch_catalog::persistence::podcast_episode::PodcastEpisodeEntity as PodcastEpisode;
use podfetch_media::naming::{NamingRules, Replacement};
use podfetch_storage::FileHandleWrapper;
use podfetch_storage::FileRequest;
use podfetch_storage::{FileType, build_podcast_image_paths, determine_file_extension};
use rss::Channel;
use tokio::task::spawn_blocking;

/// Files of a podcast, see [`FileService::collect_podcast_files`].
pub struct PodcastFiles {
    pub podcast: podfetch_storage::PodcastFileInfo,
    pub episodes: Vec<podfetch_storage::EpisodeFileInfo>,
}

#[derive(Clone)]
pub struct FileService {}

impl FileService {
    /// Whether the podcast's own cover is on disk already (its stored image
    /// URL is a local path that exists).
    pub fn check_if_podcast_main_image_downloaded(podcast_id: &str) -> bool {
        let podcast = PodcastService::get_podcast_by_directory_id(podcast_id).unwrap_or_else(|e| {
            tracing::error!("Could not load podcast {podcast_id}: {e}");
            None
        });
        match podcast {
            Some(podcast) => {
                if !podcast.image_url.contains("http") {
                    return Path::new(&podcast.image_url).exists();
                }
            }
            None => {
                return false;
            }
        }
        false
    }

    pub fn create_podcast_root_directory_exists() -> Result<(), CustomError> {
        if !FileHandleWrapper::path_exists(
            &ENVIRONMENT_SERVICE.default_podfetch_folder.clone(),
            FileRequest::Directory,
            &ENVIRONMENT_SERVICE.default_file_handler,
        ) {
            return FileHandleWrapper::create_dir(
                &ENVIRONMENT_SERVICE.default_podfetch_folder.clone(),
                &ENVIRONMENT_SERVICE.default_file_handler,
            );
        }

        Ok(())
    }

    pub async fn create_podcast_directory_exists(
        podcast_insert_model: &PodcastInsertModel,
        channel: Option<Channel>,
    ) -> Result<String, CustomError> {
        let escaped_title =
            prepare_podcast_title_to_directory(podcast_insert_model, channel).await?;
        let escaped_path = format!("podcasts/{escaped_title}");

        if FileHandleWrapper::path_exists(
            &escaped_path,
            FileRequest::Directory,
            &ENVIRONMENT_SERVICE.default_file_handler,
        ) {
            // Check if this is a new podcast with the same name as an old one

            let podcast =
                PodcastService::get_podcast_by_directory_id(&podcast_insert_model.id.to_string())?;
            match podcast {
                Some(_) => {
                    // is the same podcast
                    Ok(escaped_path)
                }
                None => {
                    // has not been inserted into the database yet
                    let mut i = 1;
                    while Path::new(&format!("podcasts/{escaped_title}-{i}")).exists() {
                        i += 1;
                    }
                    // This is save to insert because this directory does not exist
                    FileHandleWrapper::create_dir(
                        &format!("podcasts/{escaped_title}-{i}"),
                        &ENVIRONMENT_SERVICE.default_file_handler,
                    )?;
                    Ok(format!("podcasts/{escaped_title}-{i}"))
                }
            }
        } else {
            FileHandleWrapper::create_dir(
                &escaped_path,
                &ENVIRONMENT_SERVICE.default_file_handler,
            )?;
            Ok(escaped_path)
        }
    }

    /// Download the podcast's main image to `<podcast_path>/<cover_filename>.<ext>`.
    ///
    /// `directory_id` is the podcast's feed/track id (`podcasts.directory_id`),
    /// which is what the image URL is stored by - not the podcast UUID.
    pub async fn download_podcast_image(
        podcast_path: &str,
        image_url: String,
        directory_id: &str,
        cover_filename: &str,
    ) -> Result<(), CustomError> {
        let mut image_suffix = if common_infrastructure::runtime::is_default_image_url(&image_url) {
            podfetch_media::images::default_image()
        } else {
            let image_url_cloned = image_url.clone();
            podfetch_media::images::fetch_with_suffix_async(
                &ENVIRONMENT_SERVICE,
                spawn_blocking(move || {
                    let client = reqwest::blocking::Client::new();
                    determine_file_extension(&image_url_cloned, &client, FileType::Image)
                })
                .await
                .map_err(|e| {
                    tracing::error!("Determining the image type panicked: {e}");
                    CustomError::from(CustomErrorInner::Unknown(ErrorSeverity::Error))
                })?,
                &image_url,
            )
            .await?
        };

        let file_path = build_podcast_image_paths(
            podcast_path,
            &image_suffix.0,
            cover_filename,
            PodcastEpisodeService::map_to_local_url,
        );
        FileHandleWrapper::write_file_async(
            &file_path.0,
            image_suffix.1.as_mut_slice(),
            &ENVIRONMENT_SERVICE.default_file_handler,
        )
        .await?;
        PodcastEpisodeService::update_podcast_image(directory_id, &file_path.1)?;
        Ok(())
    }

    pub fn cleanup_old_episode(episode: &PodcastEpisode) -> Result<(), CustomError> {
        tracing::info!("Cleaning up old episode: {}", episode.episode_id);

        fn check_if_file_exists(file_path: &str, file_type: &FileHandlerType) -> bool {
            FileHandleWrapper::path_exists(file_path, FileRequest::File, file_type)
        }
        let Some(location) = episode.download_location.as_deref() else {
            // Never downloaded, so there is nothing on disk to clean up.
            return Ok(());
        };
        let file_type = FileHandlerType::from_stored(location);
        for path in [&episode.file_episode_path, &episode.file_image_path]
            .into_iter()
            .flatten()
        {
            if check_if_file_exists(path, &file_type) {
                FileHandleWrapper::remove_file(path, &file_type)?;
            }
        }
        Ok(())
    }

    /// Everything needed to delete a podcast's files later, collected while
    /// its episodes are still in the database.
    pub fn collect_podcast_files(podcast: &Podcast) -> PodcastFiles {
        let podcast_uuid = uuid::Uuid::parse_str(&podcast.id).unwrap_or_else(|_| uuid::Uuid::nil());
        let episodes =
            PodcastEpisodeService::get_episodes_by_podcast_id(podcast_uuid).unwrap_or_default();
        let episodes = episodes
            .iter()
            .map(|e| podfetch_storage::EpisodeFileInfo {
                download_location: e.download_location.clone(),
                file_image_path: e.file_image_path.clone(),
                file_episode_path: e.file_episode_path.clone(),
            })
            .collect();
        let podcast = podfetch_storage::PodcastFileInfo {
            // `id` is unused by the storage layer's directory removal; the
            // podcast is identified by `directory_name`. The storage DTO still
            // types it as i32, so pass a placeholder now that ids are UUIDs.
            id: 0,
            image_url: podcast.image_url.clone(),
            directory_name: podcast.directory_name.clone(),
            download_location: podcast.download_location.clone(),
        };
        PodcastFiles { podcast, episodes }
    }

    /// Remove a podcast's directory and its episode files. A directory that is
    /// already gone counts as removed.
    pub fn remove_podcast_files(files: &PodcastFiles) -> Result<(), CustomError> {
        FileHandleWrapper::remove_dir(&files.podcast, &files.episodes)
    }
}

pub async fn prepare_podcast_title_to_directory(
    podcast: &PodcastInsertModel,
    channel: Option<Channel>,
) -> Result<String, CustomError> {
    let retrieved_settings = required_settings()?;
    // The podcast row does not exist yet at this point (this runs during
    // initial insert), so there can be no per-podcast settings override.
    let opt_podcast_settings = PodcastSettingsService::get_settings_for_podcast(uuid::Uuid::nil())?;

    let podcast = match channel {
        Some(channel) => RSSFeedParser::parse_rss_feed(channel),
        None => {
            let content = get_http_client(&ENVIRONMENT_SERVICE)
                .get(&podcast.feed_url)
                .send()
                .await
                .map_err(map_reqwest_error)?
                .bytes()
                .await
                .map_err(map_reqwest_error)?;
            let channel = Channel::read_from(&content[..]).map_err(|e| {
                CustomErrorInner::BadRequest(
                    format!("Invalid podcast feed: {e}"),
                    ErrorSeverity::Info,
                )
            })?;
            RSSFeedParser::parse_rss_feed(channel)
        }
    };

    podfetch_media::naming::podcast_directory_name(
        &naming_rules(&retrieved_settings, opt_podcast_settings.as_ref()),
        &podcast,
    )
}

/// The naming scheme in effect: the podcast's own settings when they are
/// activated, the instance-wide ones otherwise.
pub fn naming_rules(
    settings: &crate::settings::Setting,
    podcast_settings: Option<&PodcastSetting>,
) -> NamingRules {
    let (replacement, podcast_format, episode_format) = match podcast_settings {
        Some(own) if own.activated => (
            &own.replacement_strategy,
            &own.podcast_format,
            &own.episode_format,
        ),
        _ => (
            &settings.replacement_strategy,
            &settings.podcast_format,
            &settings.episode_format,
        ),
    };
    NamingRules {
        replacement: Replacement::from_setting(replacement),
        podcast_format: podcast_format.clone(),
        episode_format: episode_format.clone(),
    }
}

/// The global settings row is created at startup, so a missing one is a bug.
fn required_settings() -> Result<crate::settings::Setting, CustomError> {
    SettingsService::shared().get_settings()?.ok_or_else(|| {
        tracing::error!("Global settings are missing");
        CustomErrorInner::Unknown(ErrorSeverity::Error).into()
    })
}

#[cfg(test)]
mod tests {
    // ── check_if_podcast_main_image_downloaded integration tests ─────────

    use super::FileService;
    use podfetch_catalog::domain::podcast::NewPodcast;
    use podfetch_catalog::domain::podcast::PodcastRepository;
    use podfetch_catalog::persistence::podcast::DieselPodcastRepository;
    use podfetch_persistence::db::database;
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
        assert!(
            !FileService::check_if_podcast_main_image_downloaded("nonexistent-id"),
            "non-existent podcast → image not downloaded"
        );
    }

    #[test]
    #[serial_test::serial]
    fn main_image_not_downloaded_when_url_is_http() {
        let dir_id = format!("test-http-{}", uuid::Uuid::new_v4().simple());
        seed_podcast("https://example.com/cover.jpg", &dir_id);

        crate::test_support::tests::ensure_test_env_vars();
        let result = FileService::check_if_podcast_main_image_downloaded(&dir_id);
        cleanup_podcast(&dir_id);
        assert!(!result, "HTTP image_url → not downloaded");
    }

    #[test]
    #[serial_test::serial]
    fn main_image_not_downloaded_when_local_file_missing() {
        let dir_id = format!("test-local-missing-{}", uuid::Uuid::new_v4().simple());
        seed_podcast("/tmp/does-not-exist-928347.jpg", &dir_id);

        crate::test_support::tests::ensure_test_env_vars();
        let result = FileService::check_if_podcast_main_image_downloaded(&dir_id);
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
        let result = FileService::check_if_podcast_main_image_downloaded(&dir_id);
        cleanup_podcast(&dir_id);
        let _ = std::fs::remove_file(&tmp);
        assert!(result, "local file exists → downloaded");
    }
}
