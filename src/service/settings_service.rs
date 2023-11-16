use crate::constants::inner_constants::ERR_SETTINGS_FORMAT;
use crate::controllers::settings_controller::UpdateNameSettings;
use crate::models::settings::Setting;
use crate::utils::error::CustomError;
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
        if !update_setttings.podcast_format.contains("{}") {
            return Err(CustomError::Conflict(ERR_SETTINGS_FORMAT.to_string()));
        }
        if !update_setttings.episode_format.contains("{}") {
            return Err(CustomError::Conflict(ERR_SETTINGS_FORMAT.to_string()));
        }
        Ok(update_setttings)
    }
}
