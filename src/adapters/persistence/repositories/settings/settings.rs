use diesel::{insert_into, OptionalExtension, RunQueryDsl};
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::adapters::persistence::model::settings::settings::SettingEntity;
use crate::constants::inner_constants::DEFAULT_SETTINGS;
use crate::domain::models::settings::setting::Setting;
use crate::utils::do_retry::do_retry;
use crate::utils::error::{map_db_error, CustomError};

pub struct SettingsRepository;



impl SettingsRepository {
    pub fn get_settings() -> Result<Option<Setting>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::settings::dsl::*;

        settings
            .first::<SettingEntity>(&mut get_connection())
            .optional()
            .map_err(map_db_error)
            .map(|setting| setting.map(|s| s.into()))
    }

    pub fn update_settings(
        setting: Setting,
    ) -> Result<Setting, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::settings::dsl::*;

        let setting_entity_to_save = SettingEntity::from(setting);

        let setting_to_update = settings
            .first::<Setting>(&mut get_connection())
            .expect("Error loading settings");
        do_retry(|| {
            diesel::update(&setting_to_update)
                .set(setting_entity_to_save)
                .get_result::<SettingEntity>(&mut get_connection())
        })
            .map_err(map_db_error)
            .map(|setting| setting.into())
    }

    pub fn insert_default_settings() -> Result<(), CustomError> {
        use crate::adapters::persistence::dbconfig::schema::settings::dsl::*;
        use diesel::ExpressionMethods;
        do_retry(|| {
            insert_into(settings)
                .values((
                    id.eq(DEFAULT_SETTINGS.id),
                    auto_update.eq(DEFAULT_SETTINGS.auto_update),
                    auto_download.eq(DEFAULT_SETTINGS.auto_download),
                    auto_cleanup.eq(DEFAULT_SETTINGS.auto_cleanup),
                    auto_cleanup_days.eq(DEFAULT_SETTINGS.auto_cleanup_days),
                    podcast_prefill.eq(DEFAULT_SETTINGS.podcast_prefill),
                ))
                .execute(&mut get_connection())
        })
            .map_err(map_db_error)?;
        Ok(())
    }
}