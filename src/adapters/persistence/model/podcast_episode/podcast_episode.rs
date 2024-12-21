use crate::constants::inner_constants::DEFAULT_IMAGE_URL;
use crate::adapters::persistence::dbconfig::schema::*;
use crate::adapters::persistence::dbconfig::DBType;
use crate::models::playlist_item::PlaylistItem;
use crate::models::podcasts::Podcast;
use crate::models::user::User;
use crate::utils::do_retry::do_retry;
use crate::utils::error::{map_db_error, CustomError};
use crate::utils::time::opt_or_empty_string;
use crate::DBType as DbConnection;
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use diesel::dsl::{max, sql};
use diesel::prelude::{Identifiable, Queryable, QueryableByName, Selectable};
use diesel::query_source::Alias;
use diesel::sql_types::{Bool, Integer, Nullable, Text, Timestamp};
use diesel::AsChangeset;
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::{
    delete, insert_into, BoolExpressionMethods, JoinOnDsl, NullableExpressionMethods,
    OptionalExtension, RunQueryDsl, TextExpressionMethods,
};
use rss::{Guid, Item};
use utoipa::ToSchema;
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::domain::models::episode::episode::Episode;

#[derive(
    Queryable,
    Identifiable,
    QueryableByName,
    Selectable,
    Debug,
    PartialEq,
    Clone,
    Default,
    AsChangeset,
)]
pub struct PodcastEpisodeEntity {
    #[diesel(sql_type = Integer)]
    pub(crate) id: i32,
    #[diesel(sql_type = Integer)]
    pub(crate) podcast_id: i32,
    #[diesel(sql_type = Text)]
    pub(crate) episode_id: String,
    #[diesel(sql_type = Text)]
    pub(crate) name: String,
    #[diesel(sql_type = Text)]
    pub(crate) url: String,
    #[diesel(sql_type = Text)]
    pub(crate) date_of_recording: String,
    #[diesel(sql_type = Text)]
    pub image_url: String,
    #[diesel(sql_type = Integer)]
    pub total_time: i32,
    #[diesel(sql_type = Text)]
    pub(crate) local_url: String,
    #[diesel(sql_type = Text)]
    pub(crate) local_image_url: String,
    #[diesel(sql_type = Text)]
    pub(crate) description: String,
    #[diesel(sql_type = Text)]
    pub(crate) status: String,
    #[diesel(sql_type = Nullable<Timestamp>)]
    pub(crate) download_time: Option<NaiveDateTime>,
    #[diesel(sql_type = Text)]
    pub(crate) guid: String,
    #[diesel(sql_type = Bool)]
    pub(crate) deleted: bool,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) file_episode_path: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    pub(crate) file_image_path: Option<String>,
    #[diesel(sql_type = Bool)]
    pub (crate) episode_numbering_processed : bool,
}

impl PodcastEpisodeEntity {
    pub fn is_downloaded(&self) -> bool {
        self.status == "D"
    }
}
