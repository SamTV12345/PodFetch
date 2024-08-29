use crate::constants::inner_constants::PodcastType;
use crate::models::podcast_episode::PodcastEpisode;
use crate::models::podcasts::Podcast;


#[derive(Serialize, Deserialize)]
pub struct BroadcastMessage {
    pub type_of: PodcastType,
    pub message: String,
    pub podcast: Option<Podcast>,
    pub podcast_episodes: Option<Vec<PodcastEpisode>>,
    pub podcast_episode: Option<PodcastEpisode>,
}
