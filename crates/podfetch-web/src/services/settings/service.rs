use crate::settings::{Setting, SettingsApplicationService, UpdateNameSettings};
use common_infrastructure::error::CustomError;
use common_infrastructure::rss::PodcastParsed;
use podfetch_domain::settings::SettingRepository;
use podfetch_media::naming::{EpisodeNaming, NamingRules, Replacement};
use podfetch_persistence::adapters::SettingsRepositoryImpl;
use podfetch_persistence::db::database;
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

        let sample_episode = EpisodeNaming {
            name: "My Homelab".to_string(),
            url: "http://podigee.com/rss/123".to_string(),
            date_of_recording: "2023-12-24".to_string(),
            total_time: 1200,
            description: "My description".to_string(),
            guid: "081923123".to_string(),
        };

        // Only checks that both formats work; nothing is stored here.
        let rules = NamingRules {
            replacement: Replacement::from_setting(
                &update_settings.replacement_strategy.to_string(),
            ),
            podcast_format: update_settings.podcast_format.clone(),
            episode_format: update_settings.episode_format.clone(),
        };
        podfetch_media::naming::podcast_directory_name(&rules, &sample_podcast)?;
        podfetch_media::naming::episode_file_stem(&rules, &sample_episode, 1, None)?;
        Ok(update_settings)
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
