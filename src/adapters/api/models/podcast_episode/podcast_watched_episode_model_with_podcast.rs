use chrono::NaiveDateTime;
use utoipa::ToSchema;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcasts::Podcast;

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