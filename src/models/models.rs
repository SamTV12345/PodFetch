use crate::models::itunes_models::{Podcast, PodcastEpisode};

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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PodCastAddModel {
    pub track_id: i64,
    pub user_id: i64
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PodcastWatchedPostModel {
    pub podcast_episode_id: String,
    pub time: i64
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PodcastWatchedModel {
    pub id : i64,
    pub podcast_id: i64,
    pub episode_id: String,
    pub watched_time: i64,
    pub date: String
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PodcastWatchedEpisodeModel {
    pub id : i64,
    pub podcast_id: i64,
    pub episode_id: String,
    pub url: String,
    pub name: String,
    pub image_url: String,
    pub watched_time: i64,
    pub date: String,
    pub total_time: u64
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PodcastWatchedEpisodeModelWithPodcastEpisode {
    pub id : i64,
    pub podcast_id: i64,
    pub episode_id: String,
    pub url: String,
    pub name: String,
    pub image_url: String,
    pub watched_time: i64,
    pub date: String,
    pub total_time: u64,
    pub podcast_episode: PodcastEpisode,
    pub podcast: Podcast,
}