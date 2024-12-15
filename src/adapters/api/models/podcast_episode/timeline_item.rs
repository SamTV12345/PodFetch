use crate::adapters::api::models::podcast::podcast_dto::PodcastDto;
use crate::adapters::api::models::podcast_episode::episode_dto::EpisodeDto;
use crate::adapters::api::models::podcast_episode::podcast_episode::PodcastEpisodeDto;
use crate::domain::models::favorite::favorite::Favorite;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineItem {
    pub data: Vec<(PodcastEpisodeDto, PodcastDto, Option<EpisodeDto>, Option<Favorite>)>,
    pub total_elements: i64,
}