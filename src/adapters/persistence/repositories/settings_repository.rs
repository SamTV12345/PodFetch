use crate::utils::error::CustomError;
use podfetch_domain::settings::{Setting, SettingRepository};
use podfetch_persistence::db::Database;
use podfetch_persistence::settings::DieselSettingsRepository;

pub struct SettingsRepositoryImpl {
    inner: DieselSettingsRepository,
}

impl SettingsRepositoryImpl {
    pub fn new(database: Database) -> Self {
        Self {
            inner: DieselSettingsRepository::new(database),
        }
    }
}

impl SettingRepository for SettingsRepositoryImpl {
    type Error = CustomError;

    fn get_settings(&self) -> Result<Option<Setting>, Self::Error> {
        self.inner.get_settings().map_err(Into::into)
    }

    fn update_settings(&self, setting: Setting) -> Result<Setting, Self::Error> {
        self.inner.update_settings(setting).map_err(Into::into)
    }

    fn insert_default_settings(&self) -> Result<(), Self::Error> {
        self.inner.insert_default_settings().map_err(Into::into)
    }
}
