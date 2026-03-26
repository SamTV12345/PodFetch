use podfetch_persistence::db::database;
use podfetch_persistence::adapters::SettingsRepositoryImpl;
use podfetch_persistence::podcast_episode::PodcastEpisodeEntity as PodcastEpisode;
use crate::services::file::service::{
    perform_episode_variable_replacement, perform_podcast_variable_replacement,
};
use common_infrastructure::error::CustomError;
use common_infrastructure::rss::PodcastParsed;
use podfetch_domain::settings::SettingRepository;
use crate::settings::{Setting, SettingsApplicationService, UpdateNameSettings};
use std::sync::Arc;

#[derive(Clone)]
pub struct SettingsService {
    repository: Arc<dyn SettingRepository<Error = CustomError>>,
}

impl SettingsService {
    pub fn new(repository: Arc<dyn SettingRepository<Error = CustomError>>) -> Self {
        Self { repository }
    }

    pub fn shared() -> Self {
        Self::new(Arc::new(SettingsRepositoryImpl::new(database())))
    }

    pub fn get_settings(&self) -> Result<Option<Setting>, CustomError> {
        self.repository
            .get_settings()
            .map(|settings| settings.map(Into::into))
    }

    pub fn update_settings(&self, settings: Setting) -> Result<Setting, CustomError> {
        self.repository
            .update_settings(settings.into())
            .map(Into::into)
    }

    pub fn insert_default_settings_if_not_present(&self) -> Result<(), CustomError> {
        if self.get_settings()?.is_none() {
            self.repository.insert_default_settings()?;
        }
        Ok(())
    }

    pub fn update_name(&self, update_model: UpdateNameSettings) -> Result<Setting, CustomError> {
        let settings = self.get_settings()?.unwrap();
        Self::validate_settings(update_model.clone())?;
        let mut domain_settings: podfetch_domain::settings::Setting = settings.clone().into();
        update_model.to_domain().apply_to(&mut domain_settings);
        self.update_settings(domain_settings.into())
    }

    fn validate_settings(
        update_settings: UpdateNameSettings,
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
            description: "My description".to_string(),
            download_time: None,
            guid: "081923123".to_string(),
            deleted: false,
            file_episode_path: None,
            file_image_path: None,
            episode_numbering_processed: false,
            download_location: None,
        };

        let transient_setting = build_name_only_setting(&update_settings);
        perform_podcast_variable_replacement(transient_setting.clone(), sample_podcast, None)?;
        perform_episode_variable_replacement(transient_setting, sample_episode, None)?;
        Ok(update_settings)
    }
}

fn build_name_only_setting(update: &UpdateNameSettings) -> podfetch_domain::settings::Setting {
    podfetch_domain::settings::Setting {
        id: 0,
        auto_download: false,
        auto_update: false,
        auto_cleanup: false,
        auto_cleanup_days: 0,
        podcast_prefill: 0,
        replace_invalid_characters: update.replace_invalid_characters,
        use_existing_filename: update.use_existing_filename,
        replacement_strategy: update.replacement_strategy.to_string(),
        episode_format: update.episode_format.clone(),
        podcast_format: update.podcast_format.clone(),
        direct_paths: update.direct_paths,
    }
}

impl SettingsApplicationService for SettingsService {
    type Error = CustomError;

    fn get_settings(&self) -> Result<Option<Setting>, Self::Error> {
        self.get_settings()
    }

    fn update_settings(&self, settings: Setting) -> Result<Setting, Self::Error> {
        self.update_settings(settings)
    }

    fn update_name(&self, update: UpdateNameSettings) -> Result<Setting, Self::Error> {
        self.update_name(update)
    }
}


