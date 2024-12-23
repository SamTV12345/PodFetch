use crate::constants::inner_constants::PodcastType;
use crate::domain::models::podcast::episode::PodcastEpisode;
use crate::domain::models::podcast::podcast::Podcast;

#[derive(Serialize, Deserialize)]
pub struct BroadcastMessage {
    pub type_of: PodcastType,
    pub message: String,
    pub podcast: Option<Podcast>,
    pub podcast_episodes: Option<Vec<PodcastEpisode>>,
    pub podcast_episode: Option<PodcastEpisode>,
}
