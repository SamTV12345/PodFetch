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
}