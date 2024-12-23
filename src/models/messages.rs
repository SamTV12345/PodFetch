use crate::adapters::api::models::podcast_episode_dto::PodcastEpisodeDto;
use crate::constants::inner_constants::PodcastType;
use crate::models::podcast_dto::PodcastDto;

#[derive(Serialize, Deserialize)]
pub struct BroadcastMessage {
    pub type_of: PodcastType,
    pub message: String,
    pub podcast: Option<PodcastDto>,
    pub podcast_episodes: Option<Vec<PodcastEpisodeDto>>,
    pub podcast_episode: Option<PodcastEpisodeDto>,
}
