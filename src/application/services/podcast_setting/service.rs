use crate::adapters::persistence::repositories::podcast::podcast_setting::PodcastSettingRepositoryImpl;
use crate::domain::models::podcast::podcast_setting::PodcastSetting;
use crate::utils::error::CustomError;

pub struct PodcastSettingService;


impl PodcastSettingService {
    pub fn get_settings_of_podcast(podcast_id: i32) -> Result<Option<PodcastSetting>, CustomError> {
        PodcastSettingRepositoryImpl::get_settings(&podcast_id)
    }
}