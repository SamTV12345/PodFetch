use crate::adapters::api::models::podcast_episode_dto::PodcastEpisodeDto;
use crate::models::podcast_dto::PodcastDto;
use chrono::NaiveDateTime;
use utoipa::ToSchema;
use crate::models::episode::EpisodeDto;

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PodcastAddModel {
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

#[derive(Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PodcastWatchedEpisodeModelWithPodcastEpisode {
    pub podcast_episode: PodcastEpisodeDto,
    pub podcast: PodcastDto,
    pub episode: EpisodeDto
}
