use crate::adapters::persistence::dbconfig::db::database;
use crate::adapters::persistence::repositories::podcast_settings_repository::PodcastSettingsRepositoryImpl;
use crate::models::file_path::FilenameBuilderReturn;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcast_episode_chapter::PodcastEpisodeChapter;
use crate::models::podcasts::Podcast;
use crate::service::download_service::DownloadService;
use crate::utils::error::ErrorSeverity::Warning;
use crate::utils::error::{CustomError, CustomErrorInner};
use podfetch_domain::podcast_settings::{PodcastSetting, PodcastSettingsRepository};
use std::sync::Arc;

#[derive(Clone)]
pub struct PodcastSettingsService {
    repository: Arc<dyn PodcastSettingsRepository<Error = CustomError>>,
}

impl PodcastSettingsService {
    pub fn new(repository: Arc<dyn PodcastSettingsRepository<Error = CustomError>>) -> Self {
        Self { repository }
    }

    pub fn default_service() -> Self {
        Self::new(Arc::new(PodcastSettingsRepositoryImpl::new(database())))
    }

    pub fn get_settings_for_podcast(
        podcast_id: i32,
    ) -> Result<Option<PodcastSetting>, CustomError> {
        Self::default_service().get_settings(podcast_id)
    }

    pub fn update_settings_for_podcast(
        setting_to_insert: PodcastSetting,
    ) -> Result<PodcastSetting, CustomError> {
        Self::default_service().update_settings(setting_to_insert)
    }

    pub fn get_settings(&self, podcast_id: i32) -> Result<Option<PodcastSetting>, CustomError> {
        self.repository.get_settings(podcast_id)
    }

    pub fn update_settings(
        &self,
        setting_to_insert: PodcastSetting,
    ) -> Result<PodcastSetting, CustomError> {
        let updated_setting = self.repository.upsert_settings(setting_to_insert.clone())?;
        let available_episodes =
            PodcastEpisode::get_episodes_by_podcast_id(updated_setting.podcast_id)?;
        let podcast = Podcast::get_podcast(updated_setting.podcast_id).map_err(|_| {
            CustomError::from(CustomErrorInner::Conflict(
                "Podcast not found".to_string(),
                Warning,
            ))
        })?;

        for episode in available_episodes {
            if episode.download_time.is_none() {
                continue;
            }
            let file_name_builder = FilenameBuilderReturn::new(
                episode.file_episode_path.clone().unwrap(),
                episode.file_image_path.clone().unwrap(),
            );
            match DownloadService::handle_metadata_insertion(&file_name_builder, &episode, &podcast)
            {
                Ok(chapters) => {
                    for chapter in chapters {
                        if let Err(error) = PodcastEpisodeChapter::save_chapter(&chapter, &episode)
                        {
                            log::error!(
                                "Error while saving chapter for episode {}: {}",
                                episode.id,
                                error
                            );
                        }
                    }
                }
                Err(error) => {
                    log::error!(
                        "Error while updating metadata for episode {}: {}",
                        episode.id,
                        error.inner
                    );
                }
            }
        }

        Ok(updated_setting)
    }
}
