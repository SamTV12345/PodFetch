use crate::models::settings::Setting;

pub static ITUNES_URL: &str = "https://itunes.apple.com/search?term=";

#[derive(Serialize, Deserialize, Debug)]
pub enum PodcastType {
    AddPodcast,
    AddPodcastEpisode,
    AddPodcastEpisodes,
}

pub const DEFAULT_SETTINGS: Setting = Setting {
    id: 1,
    auto_download: true,
    auto_update: true,
    auto_cleanup: true,
    auto_cleanup_days: 30,
};