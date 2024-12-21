use diesel::{OptionalExtension, QueryDsl, RunQueryDsl};
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::adapters::persistence::model::podcast_episode::podcast_setting::PodcastSettingEntity;
use crate::domain::models::podcast::podcast_setting::PodcastSetting;
use crate::utils::error::{map_db_error, CustomError};

pub struct PodcastSettingRepositoryImpl;


impl PodcastSettingRepositoryImpl {
    pub fn get_settings(id: &i32) -> Result<Option<PodcastSetting>,
        CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_settings::dsl::*;
        use diesel::ExpressionMethods;

        podcast_settings
            .filter(podcast_id.eq(id))
            .first::<PodcastSettingEntity>(&mut get_connection())
            .optional()
            .map_err(map_db_error)
            .map(|res| res.map(|res| res.into()))
    }

    pub fn update_settings(
        setting_to_insert: &PodcastSetting,
    ) -> Result<PodcastSetting, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_settings::dsl::*;
        let opt_setting = Self::get_settings(&setting_to_insert.podcast_id)?;

        match opt_setting {
            Some(_) => {
                diesel::update(podcast_settings.find(&setting_to_insert.podcast_id))
                    .set(setting_to_insert.into())
                    .execute(&mut get_connection())
                    .map_err(map_db_error)?;
            }
            None => {
                diesel::insert_into(podcast_settings)
                    .values(setting_to_insert.into())
                    .execute(&mut get_connection())
                    .map_err(map_db_error)?;
            }
        }
        // Safe because we just inserted the setting
        let setting_cloned = Self::get_settings(&setting_to_insert.podcast_id)?.unwrap();
        Ok(setting_cloned)
    }
}