
pub static ITUNES_URL: &str = "https://itunes.apple.com/search?term=";

#[derive(Serialize, Deserialize, Debug)]
pub enum PodcastType {
    AddPodcast,
    AddPodcastEpisode,
    AddPodcastEpisodes,
}