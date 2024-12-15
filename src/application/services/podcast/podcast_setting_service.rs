use crate::adapters::filesystem::update_episodes::UpdateEpisodes;
use crate::adapters::persistence::repositories::podcast::podcast_setting::PodcastSettingRepositoryImpl;
use crate::application::services::podcast_episode::service::PodcastEpisodeService;
use crate::domain::models::podcast::podcast_setting::PodcastSetting;
use crate::service::rust_service::PodcastService;
use crate::utils::error::CustomError;

pub struct PodcastSettingService;

impl PodcastSettingService {
    pub fn update_settings(
        podcast_settings: PodcastSetting
    ) -> Result<PodcastSetting, CustomError> {
        PodcastSettingRepositoryImpl::update_settings(&podcast_settings)?;
        let available_episodes  = PodcastEpisodeService::get_episodes_by_podcast_id(podcast_settings
                                                                       .podcast_id)?;
        let podcast = PodcastService::get_podcast(podcast_settings.podcast_id)?;
        UpdateEpisodes::update_available_episodes(available_episodes, podcast);
        Ok(podcast_settings)
    }

    pub fn get_settings(
        settings_id: i32
    ) -> Result<Option<PodcastSetting>, CustomError> {
        PodcastSettingRepositoryImpl::get_settings(&settings_id)
    }
}