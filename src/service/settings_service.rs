use crate::controllers::settings_controller::UpdateNameSettings;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::settings::Setting;
use crate::service::file_service::{
    perform_episode_variable_replacement, perform_podcast_variable_replacement,
};
use crate::utils::error::CustomError;
use crate::utils::rss_feed_parser::PodcastParsed;
use crate::DBType as DbConnection;

#[derive(Clone)]
pub struct SettingsService {}

impl SettingsService {
    pub fn new() -> SettingsService {
        SettingsService {}
    }

    pub fn get_settings(
        &mut self,
        conn: &mut DbConnection,
    ) -> Result<Option<Setting>, CustomError> {
        Setting::get_settings(conn)
    }

    pub fn update_settings(
        &mut self,
        settings: Setting,
        conn: &mut DbConnection,
    ) -> Result<Setting, CustomError> {
        Setting::update_settings(settings, conn)
    }

    pub fn update_name(
        &mut self,
        update_model: UpdateNameSettings,
        conn: &mut DbConnection,
    ) -> Result<Setting, CustomError> {
        let mut settings_ = self.get_settings(conn)?.unwrap();
        Self::validate_settings(update_model.clone())?;

        settings_.replace_invalid_characters = update_model.replace_invalid_characters;
        settings_.use_existing_filename = update_model.use_existing_filename;
        settings_.direct_paths = update_model.direct_paths;
        settings_.replacement_strategy = update_model.replacement_strategy.to_string();
        settings_.episode_format = update_model.episode_format;
        settings_.podcast_format = update_model.podcast_format;
        self.update_settings(settings_, conn)
    }

    fn validate_settings(
        update_setttings: UpdateNameSettings,
    ) -> Result<UpdateNameSettings, CustomError> {
        let sample_podcast = PodcastParsed {
            date: "2022-01-01".to_string(),
            summary: "A podcast about homelabing".to_string(),
            title: "The homelab podcast".to_string(),
            keywords: "computer, server, apps".to_string(),
            language: "en".to_string(),
            explicit: "false".to_string(),
        };

        let sample_episode = PodcastEpisode {
            id: 0,
            podcast_id: 0,
            episode_id: "2".to_string(),
            name: "My Homelab".to_string(),
            url: "http://podigee.com/rss/123".to_string(),
            date_of_recording: "2023-12-24".to_string(),
            image_url: "http://podigee.com/rss/123/image".to_string(),
            total_time: 1200,
            local_url: "http://localhost:8912/podcasts/123".to_string(),
            local_image_url: "http://localhost:8912/podcasts/123/image".to_string(),
            description: "My description".to_string(),
            status: "D".to_string(),
            download_time: None,
            guid: "081923123".to_string(),
            deleted: false,
            file_episode_path: None,
            file_image_path: None,
            episode_numbering_processed: false,
        };

        perform_podcast_variable_replacement(
            update_setttings.clone().into(),
            sample_podcast.clone(),
        )?;
        perform_episode_variable_replacement(
            update_setttings.clone().into(),
            sample_episode.clone(),
        )?;
        Ok(update_setttings)
    }
}
