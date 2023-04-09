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

pub const ERROR_LOGIN_MESSAGE: &str = "User either not found or password is incorrect";

pub const TELEGRAM_BOT_TOKEN: &str = "TELEGRAM_BOT_TOKEN";
pub const TELEGRAM_BOT_CHAT_ID: &str = "TELEGRAM_BOT_CHAT_ID";
pub const TELEGRAM_API_ENABLED: &str = "TELEGRAM_API_ENABLED";


// User management roles
pub const ROLE_ADMIN: &str = "admin";
pub const ROLE_UPLOADER: &str = "uploader";
pub const ROLE_USER: &str = "User";