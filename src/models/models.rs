use crate::models::itunes_models::{Podcast, PodcastEpisode};
use diesel::prelude::*;
use diesel::sql_types::{Integer, Text};
use diesel::{QueryId, Selectable};
use utoipa::ToSchema;
use chrono::NaiveDateTime;
use diesel::sql_types::Timestamp;
use crate::DbConnection;

// decode request data
#[derive(Deserialize)]
pub struct UserData {
    pub username: String,
}
// this is to insert users to database
#[derive(Serialize, Deserialize)]
pub struct NewUser {
    pub username: String,
    pub password: String,
    pub first_name: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PodCastAddModel {
    pub track_id: i32,
    pub user_id: i32,
}

pub struct PodcastInsertModel {
    pub title: String,
    pub id: i32,
    pub feed_url: String,
    pub image_url: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PodcastWatchedPostModel {
    pub podcast_episode_id: String,
    pub time: i32,
}



#[derive(Serialize, Deserialize, Queryable, QueryableByName, Clone, ToSchema, QueryId,
Selectable, Debug)]
#[serde(rename_all = "camelCase")]
#[diesel(table_name=crate::dbconfig::schema::podcast_history_items)]
pub struct PodcastHistoryItem {
    #[diesel(sql_type = Integer, column_name=id)]
    pub id: i32,
    #[diesel(sql_type = Integer, column_name=podcast_id)]
    pub podcast_id: i32,
    #[diesel(sql_type = Text,column_name=episode_id)]
    pub episode_id: String,
    #[diesel(sql_type = Integer, column_name=watched_time)]
    pub watched_time: i32,
    #[diesel(sql_type = Timestamp,column_name=date)]
    pub date: NaiveDateTime,
    #[diesel(sql_type = Text,column_name=username)]
    pub username: String
}

impl PodcastHistoryItem{
    pub fn delete_by_username(username1: String, conn: &mut DbConnection) -> Result<(),
        diesel::result::Error>{
        use crate::dbconfig::schema::podcast_history_items::dsl::*;
        diesel::delete(podcast_history_items.filter(username.eq(username1)))
            .execute(conn)?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PodcastWatchedEpisodeModel {
    pub id: i32,
    pub podcast_id: i32,
    pub episode_id: String,
    pub url: String,
    pub name: String,
    pub image_url: String,
    pub watched_time: i32,
    pub date: String,
    pub total_time: i32,
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PodcastWatchedEpisodeModelWithPodcastEpisode {
    pub id: i32,
    pub podcast_id: i32,
    pub episode_id: String,
    pub url: String,
    pub name: String,
    pub image_url: String,
    pub watched_time: i32,
    pub date: NaiveDateTime,
    pub total_time: i32,
    pub podcast_episode: PodcastEpisode,
    pub podcast: Podcast,
}

#[derive(Serialize, Deserialize, Queryable,Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    pub id: i32,
    pub type_of_message: String,
    pub message: String,
    pub created_at: String,
    pub status: String,
}
