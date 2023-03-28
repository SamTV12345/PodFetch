use crate::models::itunes_models::{Podcast, PodcastEpisode};
use diesel::prelude::*;
use diesel::sql_types::{Integer, Text};
use utoipa::{ToSchema};

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
    pub user_id: i32
}


pub struct PodcastInsertModel {
    pub title: String,
    pub id: i32,
    pub feed_url: String,
    pub image_url: String
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PodcastWatchedPostModel {
    pub podcast_episode_id: String,
    pub time: i32
}

#[derive(Serialize, Deserialize, Queryable, QueryableByName, Clone, ToSchema )]
#[serde(rename_all = "camelCase")]
pub struct PodcastHistoryItem {
    #[diesel(sql_type = Integer)]
    pub id : i32,
    #[diesel(sql_type = Integer)]
    pub podcast_id: i32,
    #[diesel(sql_type = Text)]
    pub episode_id: String,
    #[diesel(sql_type = Integer)]
    pub watched_time: i32,
    #[diesel(sql_type = Text)]
    pub date: String
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PodcastWatchedEpisodeModel {
    pub id : i32,
    pub podcast_id: i32,
    pub episode_id: String,
    pub url: String,
    pub name: String,
    pub image_url: String,
    pub watched_time: i32,
    pub date: String,
    pub total_time: i32
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PodcastWatchedEpisodeModelWithPodcastEpisode {
    pub id : i32,
    pub podcast_id: i32,
    pub episode_id: String,
    pub url: String,
    pub name: String,
    pub image_url: String,
    pub watched_time: i32,
    pub date: String,
    pub total_time: i32,
    pub podcast_episode: PodcastEpisode,
    pub podcast: Podcast,
}

#[derive(Serialize, Deserialize,Queryable, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    pub id: i32,
    pub type_of_message: String,
    pub message: String,
    pub created_at: String,
    pub status: String
}