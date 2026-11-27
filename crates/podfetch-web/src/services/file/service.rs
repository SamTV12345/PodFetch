use podfetch_persistence::podcast::PodcastEntity as Podcast;

use std::path::Path;

use crate::podcast::PodcastInsertModel;
use crate::podcast_settings::PodcastSetting;
use crate::services::download::service::DownloadService;
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
use podfetch_media::naming::{EpisodeNaming, NamingRules, Replacement};
use podfetch_persistence::db::DBType as DbConnection;
use podfetch_persistence::podcast_episode::PodcastEpisodeEntity as PodcastEpisode;
use podfetch_storage::FileHandleWrapper;
use podfetch_storage::{FileRequest, resolve_file_handler_type};
use podfetch_storage::{
    FileType, build_podcast_image_paths, create_available_directory, determine_file_extension,
};
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
    pub fn check_if_podcast_main_image_downloaded(
        podcast_id: &str,
        _conn: &mut DbConnection,
    ) -> bool {
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
        let mut image_suffix = if DownloadService::is_default_fallback_image_url(&image_url) {
            DownloadService::get_default_fallback_image_data()
        } else {
            let image_url_cloned = image_url.clone();
            DownloadService::handle_suffix_response_async(
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

    pub fn ensure_podcast_episode_directory_available(
        base_path: &str,
        podcast: &Podcast,
    ) -> Result<String, CustomError> {
        let download_location = resolve_file_handler_type(podcast.download_location.clone());

        create_available_directory(
            base_path,
            |path| FileHandleWrapper::path_exists(path, FileRequest::Directory, &download_location),
            |path| FileHandleWrapper::path_exists(path, FileRequest::NoopS3, &download_location),
            |path| FileHandleWrapper::create_dir(path, &download_location),
        )
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

/// What the episode's file is named after.
pub fn episode_naming(episode: &PodcastEpisode) -> EpisodeNaming {
    EpisodeNaming {
        name: episode.name.clone(),
        date_of_recording: episode.date_of_recording.clone(),
        guid: episode.guid.clone(),
        url: episode.url.clone(),
        description: episode.description.clone(),
        total_time: episode.total_time,
    }
}

pub fn prepare_podcast_episode_title_to_directory(
    podcast_episode: PodcastEpisode,
) -> Result<String, CustomError> {
    let retrieved_settings = required_settings()?;
    if retrieved_settings.use_existing_filename
        && let Some(filename) = podfetch_media::naming::filename_of_url(&podcast_episode.url)
    {
        return Ok(filename);
    }
    let podcast_uuid = uuid::Uuid::parse_str(&podcast_episode.podcast_id)
        .map_err(|_| CustomError::from(CustomErrorInner::NotFound(ErrorSeverity::Warning)))?;
    let podcast_settings = PodcastSettingsService::get_settings_for_podcast(podcast_uuid)?;
    let episode_number =
        crate::usecases::podcast_episode::PodcastEpisodeUseCase::get_position_of_episode(
            &podcast_episode.date_of_recording,
            podcast_uuid,
        )? as i64;
    let total_episode_count =
        crate::usecases::podcast_episode::PodcastEpisodeUseCase::get_total_episode_count(
            podcast_uuid,
        )?;
    podfetch_media::naming::episode_file_stem(
        &naming_rules(&retrieved_settings, podcast_settings.as_ref()),
        &episode_naming(&podcast_episode),
        episode_number,
        Some(total_episode_count),
    )
}

/// The global settings row is created at startup, so a missing one is a bug.
fn required_settings() -> Result<crate::settings::Setting, CustomError> {
    SettingsService::shared().get_settings()?.ok_or_else(|| {
        tracing::error!("Global settings are missing");
        CustomErrorInner::Unknown(ErrorSeverity::Error).into()
    })
}
