use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::adapters::persistence::dbconfig::schema::podcast_settings;
use crate::models::file_path::FilenameBuilderReturn;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcasts::Podcast;
use crate::service::download_service::DownloadService;
use crate::utils::error::{map_db_error, CustomError, CustomErrorInner};
use diesel::{
    AsChangeset, Identifiable, Insertable, OptionalExtension, QueryDsl, Queryable, RunQueryDsl,
};
use utoipa::ToSchema;

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
    pub direct_paths: bool,
    #[diesel(sql_type = Bool)]
    pub activated: bool,
    #[diesel(sql_type = Integer)]
    pub podcast_prefill: i32,
}

impl PodcastSetting {
    pub fn get_settings(id: i32) -> Result<Option<PodcastSetting>, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_settings::dsl::*;
        use diesel::ExpressionMethods;

        podcast_settings
            .filter(podcast_id.eq(id))
            .first::<PodcastSetting>(&mut get_connection())
            .optional()
            .map_err(map_db_error)
    }

    pub fn handle_episode_numbering() {}

    pub fn update_settings(
        setting_to_insert: &PodcastSetting,
    ) -> Result<PodcastSetting, CustomError> {
        use crate::adapters::persistence::dbconfig::schema::podcast_settings::dsl::*;
        let opt_setting = Self::get_settings(setting_to_insert.podcast_id)?;

        match opt_setting {
            Some(_) => {
                diesel::update(podcast_settings.find(setting_to_insert.podcast_id))
                    .set(setting_to_insert.clone())
                    .execute(&mut get_connection())
                    .map_err(map_db_error)?;
            }
            None => {
                diesel::insert_into(podcast_settings)
                    .values(setting_to_insert.clone())
                    .execute(&mut get_connection())
                    .map_err(map_db_error)?;
            }
        }
        let available_episodes =
            PodcastEpisode::get_episodes_by_podcast_id(setting_to_insert.podcast_id)?;
        let podcast = Podcast::get_podcast(setting_to_insert.podcast_id);
        if podcast.is_err() {
            return Err(CustomErrorInner::Conflict("Podcast not found".to_string()).into());
        }
        let podcast = podcast?;
        for e in available_episodes {
            if e.download_time.is_some() {
                let f_e = e.clone();
                let file_name_builder = FilenameBuilderReturn::new(
                    f_e.file_episode_path.unwrap(),
                    f_e.file_image_path.unwrap(),
                );
                let result = DownloadService::handle_metadata_insertion(
                    &file_name_builder,
                    &e.clone(),
                    &podcast,
                );
                if result.is_err() {
                    log::error!("Error while updating metadata for episode: {}", e.id);
                }
            }
        }
        Ok(setting_to_insert.clone())
    }
}
