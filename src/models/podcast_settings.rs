use diesel::{AsChangeset, Identifiable, Insertable, OptionalExtension, QueryDsl, Queryable, RunQueryDsl};
use utoipa::ToSchema;
use crate::DBType;
use crate::utils::error::{map_db_error, CustomError};
use crate::dbconfig::schema::podcast_settings;

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
#[diesel(primary_key(podcast_id))]
pub struct PodcastSetting {
    #[diesel(sql_type = Integer)]
    pub podcast_id: i32,
    #[diesel(sql_type = Bool)]
    pub episode_numbering: bool,
    #[diesel(sql_type = Bool)]
    pub auto_download: bool,
    #[diesel(sql_type = Bool)]
    pub auto_update: bool,
    #[diesel(sql_type = Bool)]
    pub auto_cleanup: bool,
    #[diesel(sql_type = Integer)]
    pub auto_cleanup_days: i32,
    #[diesel(sql_type = Bool)]
    pub replace_invalid_characters: bool,
    #[diesel(sql_type = Bool)]
    pub use_existing_filename: bool,
    #[diesel(sql_type = Text)]
    pub replacement_strategy: String,
    #[diesel(sql_type = Text)]
    pub episode_format: String,
    #[diesel(sql_type = Text)]
    pub podcast_format: String,
    #[diesel(sql_type = Bool)]
    pub direct_paths: bool
}


impl PodcastSetting {
    pub fn get_settings(conn: &mut DBType, id: i32) -> Result<Option<PodcastSetting>,
        CustomError> {
        use crate::dbconfig::schema::podcast_settings::dsl::*;
        use diesel::ExpressionMethods;

        podcast_settings
            .filter(podcast_id.eq(id))
            .first::<PodcastSetting>(conn)
            .optional()
            .map_err(map_db_error)
    }

    pub fn update_settings(
        setting: &PodcastSetting,
        conn: &mut DBType,
    ) -> Result<PodcastSetting, CustomError> {
        use crate::dbconfig::schema::podcast_settings::dsl::*;
        let opt_setting = Self::get_settings(conn, setting.podcast_id)?;

        match opt_setting {
            Some(_) => {
                diesel::update(podcast_settings.find(setting.podcast_id))
                    .set(setting.clone())
                    .get_result(conn)
                    .map_err(map_db_error)
            }
            None => {
                diesel::insert_into(podcast_settings)
                    .values(setting.clone())
                    .get_result(conn)
                    .map_err(map_db_error)
            }
        }

    }
}