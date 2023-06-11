
use crate::constants::constants::ERR_SETTINGS_FORMAT;
use crate::controllers::settings_controller::{UpdateNameSettings};
use crate::db::DB;
use crate::DbConnection;
use crate::models::settings::Setting;

#[derive(Clone)]
pub struct SettingsService{
}

impl SettingsService{
    pub fn new() -> SettingsService{
        SettingsService{
        }
    }

    pub fn get_settings(&mut self, mut db:DB, conn: &mut DbConnection) -> Option<Setting> {
        Setting::get_settings(conn)
    }

    pub fn update_settings(&mut self, settings: Setting, mut db:DB,conn: &mut DbConnection) -> Setting{
        Setting::update_settings(settings, conn)
    }

    pub fn update_name(&mut self, update_model: UpdateNameSettings, db:DB, conn: &mut DbConnection) ->
                                                                                Result<Setting,String>{
        let mut settings_ = self.get_settings(db.clone(), conn).unwrap();
        let res = Self::validate_settings(update_model.clone());
        if res.is_err(){
            return  Err(res.err().unwrap());
        }

        settings_.replace_invalid_characters = update_model.replace_invalid_characters;
        settings_.use_existing_filename = update_model.use_existing_filenames;
        settings_.replacement_strategy = update_model.replacement_strategy.to_string();
        settings_.episode_format = update_model.episode_format;
        settings_.podcast_format = update_model.podcast_format;
        Ok(self.update_settings(settings_,db.clone(), conn))
    }


    fn validate_settings(update_setttings: UpdateNameSettings)->Result<UpdateNameSettings, String>{
        if !update_setttings.podcast_format.contains("{}"){
            return Err(ERR_SETTINGS_FORMAT.to_string())
        }
        if !update_setttings.episode_format.contains("{}"){
            return Err(ERR_SETTINGS_FORMAT.to_string())
        }
        return Ok(update_setttings)
    }
}