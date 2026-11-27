use crate::podcast_settings::PodcastSetting;
use crate::usecases::podcast_episode::PodcastEpisodeUseCase as PodcastEpisodeService;
use common_infrastructure::error::ErrorSeverity::Warning;
use common_infrastructure::error::{CustomError, CustomErrorInner};
use podfetch_domain::podcast_settings::PodcastSettingsRepository;
use podfetch_persistence::adapters::PodcastSettingsRepositoryImpl;
use podfetch_persistence::db::database;
use std::sync::Arc;
use uuid::Uuid;

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
        podcast_id: Uuid,
    ) -> Result<Option<PodcastSetting>, CustomError> {
        Self::default_service().get_settings(podcast_id)
    }

    pub fn update_settings_for_podcast(
        setting_to_insert: PodcastSetting,
    ) -> Result<PodcastSetting, CustomError> {
        Self::default_service().update_settings(setting_to_insert)
    }

    pub fn get_settings(&self, podcast_id: Uuid) -> Result<Option<PodcastSetting>, CustomError> {
        self.repository
            .get_settings(podcast_id)
            .map(|setting| setting.map(Into::into))
    }

    pub fn update_settings(
        &self,
        setting_to_insert: PodcastSetting,
    ) -> Result<PodcastSetting, CustomError> {
        let updated_setting = self
            .repository
            .upsert_settings(setting_to_insert.clone().into())?;
        let podcast_uuid = updated_setting.podcast_id;
        let available_episodes = PodcastEpisodeService::get_episodes_by_podcast_id(podcast_uuid)?;
        let podcast = crate::services::podcast::service::PodcastService::get_podcast(podcast_uuid)
            .map_err(|_| {
                CustomError::from(CustomErrorInner::Conflict(
                    "Podcast not found".to_string(),
                    Warning,
                ))
            })?;

        let media = crate::media::services();
        let podcast = crate::media::media_podcast(&podcast);
        for episode in available_episodes {
            if episode.download_time.is_none() {
                continue;
            }
            let episode = crate::media::media_episode(&episode);
            if let Err(error) = media.downloader.retag(&episode, &podcast) {
                tracing::error!(
                    "Error while updating metadata for episode {}: {}",
                    episode.id,
                    error.inner
                );
            }
            if let Some(audio_path) = episode
                .file
                .file_episode_path
                .as_deref()
                .filter(|p| !p.is_empty())
            {
                media
                    .nfo
                    .regenerate_for_episode(&podcast, &episode, audio_path);
            }
        }

        media.nfo.ensure_cover_filename(&podcast);
        Ok(updated_setting.into())
    }
}
