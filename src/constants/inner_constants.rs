use crate::service::environment_service::EnvironmentService;
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;
use std::sync::LazyLock;

pub static ITUNES_URL: &str = "https://itunes.apple.com/search";

#[derive(Serialize, Deserialize, Debug)]
pub enum PodcastType {
    AddPodcast,
    AddPodcastEpisode,
    AddPodcastEpisodes,
    DeletePodcastEpisode,
    RefreshPodcast,
    OpmlAdded,
    OpmlErrored,
}

pub const DEFAULT_SETTINGS: PartialSettings = PartialSettings {
    id: 1,
    auto_download: true,
    auto_update: true,
    auto_cleanup: true,
    auto_cleanup_days: 30,
    podcast_prefill: 5,
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
pub const PODFETCH_FOLDER: &str = "PODFETCH_FOLDER";
pub const DEFAULT_PODFETCH_FOLDER: &str = "podcasts";
pub const FILE_HANDLER: &str = "FILE_HANDLER";

use crate::models::episode::Episode;
use crate::models::favorite_podcast_episode::FavoritePodcastEpisode;
use crate::models::podcast_episode::PodcastEpisode;
use crate::service::logging_service::init_logging;
use crate::utils::error::CustomError;
use utoipa::ToSchema;

// User management roles
#[derive(Serialize, Deserialize, Debug, PartialEq, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    Uploader,
    User,
}

impl fmt::Display for Role {
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

impl FromStr for Role {
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
impl Role {
    pub const VALUES: [Self; 3] = [Self::User, Self::Admin, Self::Uploader];
}

// environment keys
pub const OIDC_AUTH: &str = "OIDC_AUTH";
pub const OIDC_REDIRECT_URI: &str = "OIDC_REDIRECT_URI";
pub const OIDC_AUTHORITY: &str = "OIDC_AUTHORITY";
pub const OIDC_CLIENT_ID: &str = "OIDC_CLIENT_ID";
pub const OIDC_SCOPE: &str = "OIDC_SCOPE";

pub const BASIC_AUTH: &str = "BASIC_AUTH";

pub const USERNAME: &str = "USERNAME";
pub const PASSWORD: &str = "PASSWORD";
pub const API_KEY: &str = "API_KEY";
pub const SERVER_URL: &str = "SERVER_URL";

pub const SUB_DIRECTORY: &str = "SUB_DIRECTORY";

pub const POLLING_INTERVAL: &str = "POLLING_INTERVAL";

pub const STANDARD_USER: &str = "user123";

pub const PODCAST_FILENAME: &str = "podcast";
pub const PODCAST_IMAGENAME: &str = "image";

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

pub const COMMON_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
(KHTML, like Gecko) Chrome/116.0.0.0 Safari/537.36";

pub const OIDC_JWKS: &str = "OIDC_JWKS";

// Default device when viewing via web interface
pub const DEFAULT_DEVICE: &str = "webview";

// static constants

pub static ENVIRONMENT_SERVICE: LazyLock<EnvironmentService> = LazyLock::new(|| {
    init_logging();
    #[cfg(test)]
    {
        let mut env = EnvironmentService::new();
        #[cfg(feature = "sqlite")]
        {
            env.database_url = "sqlite://./podcast.db".to_string();
        }
        #[cfg(feature = "postgresql")]
        {
            env.database_url = "postgres://postgres:postgres@127.0.0.1:55002/postgres".to_string();
        }
        println!("Environment: {:?}", env.database_url);
        env.http_basic = true;
        env.username = Some("postgres".to_string());
        env.password =
            Some("a942b37ccfaf5a813b1432caa209a43b9d144e47ad0de1549c289c253e556cd5".to_string());
        env.gpodder_integration_enabled = true;
        env
    }
    #[cfg(not(test))]
    EnvironmentService::new()
});

pub static DEFAULT_IMAGE_URL: &str = "ui/default.jpg";
pub static ITUNES: &str = "itunes";

// Reverse proxy headers
pub const REVERSE_PROXY: &str = "REVERSE_PROXY";
pub const REVERSE_PROXY_HEADER: &str = "REVERSE_PROXY_HEADER";
pub const REVERSE_PROXY_AUTO_SIGN_UP: &str = "REVERSE_PROXY_AUTO_SIGN_UP";
pub const PODFETCH_PROXY_FOR_REQUESTS: &str = "PODFETCH_PROXY";

pub const MAIN_ROOM: &str = "main";
#[cfg(feature = "postgresql")]
pub const CONNECTION_NUMBERS: &str = "DB_CONNECTIONS";

pub type PodcastEpisodeWithFavorited = Result<
    Vec<(
        PodcastEpisode,
        Option<Episode>,
        Option<FavoritePodcastEpisode>,
    )>,
    CustomError,
>;

// S3 configuration
pub const S3_URL: &str = "S3_URL";
pub const S3_REGION: &str = "S3_REGION";
pub const S3_ACCESS_KEY: &str = "S3_ACCESS_KEY";
pub const S3_SECRET_KEY: &str = "S3_SECRET_KEY";
pub const S3_PROFILE: &str = "S3_PROFILE";
pub const S3_SECURITY_TOKEN: &str = "S3_SECURITY_TOKEN";
pub const S3_SESSION_TOKEN: &str = "S3_SESSION_TOKEN";
