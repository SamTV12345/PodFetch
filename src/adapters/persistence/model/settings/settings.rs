use crate::constants::inner_constants::DEFAULT_SETTINGS;
use crate::adapters::persistence::dbconfig::schema::*;
use crate::service::environment_service::OidcConfig;
use crate::utils::do_retry::do_retry;
use crate::utils::error::{map_db_error, CustomError};
use diesel::insert_into;
use diesel::prelude::{AsChangeset, Identifiable, Insertable, Queryable};
use diesel::{OptionalExtension, RunQueryDsl};
use utoipa::ToSchema;
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::domain::models::settings::setting::Setting;

#[derive(
    Serialize,
    Deserialize,
    Queryable,
    Insertable,
    Debug,
    Clone,
    Identifiable,
    AsChangeset,
    ToSchema,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct SettingEntity {
    pub id: i32,
    pub auto_download: bool,
    pub auto_update: bool,
    pub auto_cleanup: bool,
    pub auto_cleanup_days: i32,
    pub podcast_prefill: i32,
    pub replace_invalid_characters: bool,
    pub use_existing_filename: bool,
    pub replacement_strategy: String,
    pub episode_format: String,
    pub podcast_format: String,
    pub direct_paths: bool
}


impl From<Setting> for SettingEntity {
    fn from(val: Setting) -> Self {
        SettingEntity {
            id: val.id,
            auto_download: val.auto_download,
            auto_update: val.auto_update,
            use_existing_filename: val.use_existing_filename,
            replace_invalid_characters: val.replace_invalid_characters,
            replacement_strategy: val.replacement_strategy,
            episode_format: val.episode_format,
            podcast_format: val.podcast_format,
            direct_paths: val.direct_paths,
            auto_cleanup_days: val.auto_cleanup_days,
            auto_cleanup: val.auto_cleanup,
            podcast_prefill: val.podcast_prefill,
        }
    }
}

impl Into<Setting> for SettingEntity {
    fn into(self) -> Setting {
        Setting {
            id: self.id,
            auto_download: self.auto_download,
            auto_update: self.auto_update,
            use_existing_filename: self.use_existing_filename,
            replace_invalid_characters: self.replace_invalid_characters,
            replacement_strategy: self.replacement_strategy,
            episode_format: self.episode_format,
            podcast_format: self.podcast_format,
            direct_paths: self.direct_paths,
            auto_cleanup_days: self.auto_cleanup_days,
            auto_cleanup: self.auto_cleanup,
            podcast_prefill: self.podcast_prefill,
        }
    }
}




