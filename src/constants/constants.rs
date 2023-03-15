
pub static ITUNES_URL: &str = "https://itunes.apple.com/search?term=";


pub static ADD_PODCAST_TYPE: &str = "AddPodcast";
pub static ADD_PODCAST_EPISODE_TYPE: &str = "AddPodcastEpisode";
pub static ADD_PODCAST_EPISODES_TYPE: &str = "AddPodcastEpisodes";


pub enum PodcastType {
    AddPodcast,
    AddPodcastEpisode,
    AddPodcastEpisodes,
}