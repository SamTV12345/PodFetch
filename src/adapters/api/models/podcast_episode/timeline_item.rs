use crate::adapters::api::models::favorite::favorite::FavoriteDto;
use crate::adapters::api::models::podcast::podcast_dto::PodcastDto;
use crate::adapters::api::models::podcast_episode::episode_dto::EpisodeDto;
use crate::adapters::api::models::podcast_episode::podcast_episode::PodcastEpisodeDto;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineItem {
    pub data: Vec<(PodcastEpisodeDto, PodcastDto, Option<EpisodeDto>, Option<FavoriteDto>)>,
    pub total_elements: i64,
}