use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

pub static ITUNES_URL: &str = "https://itunes.apple.com/search?term=";

#[derive(Serialize, Deserialize, Debug)]
pub enum PodcastType {
    AddPodcast,
    AddPodcastEpisode,
    AddPodcastEpisodes,
    DeletePodcastEpisode,
    RefreshPodcast,
    OpmlAdded,
    OpmlErrored
}

pub const DEFAULT_SETTINGS: PartialSettings = PartialSettings {
    id: 1,
    auto_download: true,
    auto_update: true,
    auto_cleanup: true,
    auto_cleanup_days: 30,
    podcast_prefill: 5
};


pub struct PartialSettings {
    pub id: i32,
    pub auto_download: bool,
    pub auto_update: bool,
    pub auto_cleanup: bool,
    pub auto_cleanup_days: i32,
    pub podcast_prefill: i32,
}

pub const TELEGRAM_BOT_TOKEN: &str = "TELEGRAM_BOT_TOKEN";
pub const TELEGRAM_BOT_CHAT_ID: &str = "TELEGRAM_BOT_CHAT_ID";
pub const TELEGRAM_API_ENABLED: &str = "TELEGRAM_API_ENABLED";


// User management roles
#[derive( Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    Uploader,
    User,
}


impl fmt::Display for Role{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Role::Admin => {
                write!(f, "admin")
            }
            Role::Uploader => {
                write!(f, "uploader")
            }
            Role::User => {
                write!(f, "user")
            }
        }
    }
}

impl FromStr for Role{
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "admin" => Ok(Role::Admin),
            "uploader" => Ok(Role::Uploader),
            "user" => Ok(Role::User),
            _ => Err(()),
        }
    }
}
impl Role{
    pub const VALUES: [Self; 3] = [Self::User, Self::Admin, Self::Uploader];
}

// environment keys
pub const OIDC_AUTH:&str = "OIDC_AUTH";
pub const OIDC_REDIRECT_URI:&str = "OIDC_REDIRECT_URI";
pub const OIDC_AUTHORITY:&str = "OIDC_AUTHORITY";
pub const OIDC_CLIENT_ID:&str = "OIDC_CLIENT_ID";
pub const OIDC_SCOPE:&str = "OIDC_SCOPE";

pub const BASIC_AUTH:&str = "BASIC_AUTH";


pub const USERNAME:&str = "USERNAME";
pub const PASSWORD:&str = "PASSWORD";

pub const SERVER_URL: &str = "SERVER_URL";

pub const SUB_DIRECTORY: &str = "SUB_DIRECTORY";

pub const POLLING_INTERVAL: &str = "POLLING_INTERVAL";

pub const STANDARD_USER: &str = "user123";


pub const ERR_SETTINGS_FORMAT: &str = "A podcast/episode format needs to contain an opening and \
closing bracket ({}).";

pub const PODCAST_FILENAME: &str = "podcast";
pub const PODCAST_IMAGENAME:&str = "image";

pub const POLLING_INTERVAL_DEFAULT: u32 = 300;


// podindex config

pub const PODINDEX_API_KEY: &str = "PODINDEX_API_KEY";
pub const PODINDEX_API_SECRET: &str = "PODINDEX_API_SECRET";


// GPodder config

pub const GPODDER_INTEGRATION_ENABLED: &str = "GPODDER_INTEGRATION_ENABLED";

pub const DATABASE_URL: &str = "DATABASE_URL";

pub const DATABASE_URL_DEFAULT_SQLITE: &str = "sqlite://./db/podcast.db";


pub const CSS: &str = "css";
pub const JS: &str = "javascript";


pub const MAX_FILE_TREE_DEPTH:i32 = 4;


pub const COMMON_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
(KHTML, like Gecko) Chrome/116.0.0.0 Safari/537.36";

pub const OIDC_JWKS: &str = "OIDC_JWKS";