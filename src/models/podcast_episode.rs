use crate::dbconfig::schema::*;
use chrono::NaiveDateTime;
use diesel::prelude::{Queryable, Identifiable, Selectable, QueryableByName};
use diesel::{RunQueryDsl};
use utoipa::ToSchema;
use diesel::sql_types::{Integer, Text, Nullable, Bool, Timestamp};
use diesel::QueryDsl;
use diesel::ExpressionMethods;
use crate::DbConnection;

#[derive(Queryable, Identifiable,QueryableByName, Selectable, Debug, PartialEq, Clone, ToSchema,
Serialize, Deserialize, Default)]
pub struct PodcastEpisode {
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
    pub(crate) download_time: Option<NaiveDateTime>
}

impl PodcastEpisode{
    pub fn is_downloaded(&self) -> bool{
        self.status == "D"
    }
}