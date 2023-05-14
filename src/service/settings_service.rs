use crate::controllers::settings_controller::{UpdateNameSettings};
use crate::db::DB;
use crate::models::settings::Setting;

#[derive(Clone)]
pub struct SettingsService{
    db: DB
}

impl SettingsService{
    pub fn new() -> SettingsService{
        SettingsService{
            db: DB::new().expect("Error creating db")
        }
    }

    pub fn get_settings(&mut self) -> Option<Setting> {
        self.db.get_settings()
    }

    pub fn update_settings(&mut self, settings: Setting) -> Setting{
        self.db.update_settings(settings)
    }

    pub fn update_name(&mut self, update_model: UpdateNameSettings) -> Setting{
        let mut settings = self.get_settings().unwrap();

        settings.replace_invalid_characters = update_model.replace_invalid_characters;
        settings.use_existing_filename = update_model.use_existing_filenames;
        settings.replacement_strategy = update_model.replacement_strategy.to_string();
        settings.episode_format = update_model.episode_format;
        self.update_settings(settings)
    }
}