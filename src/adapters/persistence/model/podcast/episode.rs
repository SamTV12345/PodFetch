use crate::constants::inner_constants::DEFAULT_DEVICE;
use crate::adapters::persistence::dbconfig::schema::episodes;
use crate::adapters::persistence::dbconfig::schema::episodes::dsl::episodes as episodes_dsl;
use crate::DBType as DbConnection;
use chrono::{NaiveDateTime, Utc};
use diesel::sql_types::{Integer, Nullable, Text, Timestamp};
use diesel::{ExpressionMethods};
use diesel::{
    Insertable,
    QueryId, Queryable, QueryableByName, Selectable,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::domain::models::episode::episode::Episode;

#[derive(
    Serialize,
    Deserialize,
    Debug,
    Queryable,
    QueryableByName,
    Insertable,
    Clone,
    Selectable,
    ToSchema,
    QueryId,
)]
#[table_name = "episodes"]
pub struct EpisodeEntity {
    #[diesel(sql_type = Integer)]
    pub id: i32,
    #[diesel(sql_type = Text)]
    pub username: String,
    #[diesel(sql_type = Text)]
    pub device: String,
    #[diesel(sql_type = Text)]
    pub podcast: String,
    #[diesel(sql_type = Text)]
    pub episode: String,
    #[diesel(sql_type = Timestamp)]
    pub timestamp: NaiveDateTime,
    #[diesel(sql_type = Nullable<Text>)]
    pub guid: Option<String>,
    #[diesel(sql_type = Text)]
    pub action: String,
    #[diesel(sql_type = Nullable<Integer>)]
    pub started: Option<i32>,
    #[diesel(sql_type = Nullable<Integer>)]
    pub position: Option<i32>,
    #[diesel(sql_type = Nullable<Integer>)]
    pub total: Option<i32>,
}


impl From<Episode> for EpisodeEntity {
    fn from(value: Episode) -> Self {
        EpisodeEntity {
            id: value.id,
            username: value.username,
            device: value.device,
            podcast: value.podcast,
            episode: value.episode,
            timestamp: value.timestamp,
            guid: value.guid,
            action: value.action,
            started: value.started,
            position: value.position,
            total: value.total,
        }
    }
}