use std::convert::Into;
use std::fmt;
use std::fmt::Formatter;
use std::string::ToString;
use std::sync::LazyLock;
use common_infrastructure::config::EnvironmentService;
use common_infrastructure::logging::init_logging;

pub static ITUNES_URL: &str = "https://itunes.apple.com/search";

#[cfg(feature = "postgresql")]
pub use common_infrastructure::config::CONNECTION_NUMBERS;
#[allow(unused_imports)]
pub use common_infrastructure::config::{
    API_KEY, BASIC_AUTH, DATABASE_URL, DATABASE_URL_DEFAULT_SQLITE, DEFAULT_OIDC_REFRESH_INTERVAL,
    DEFAULT_PODFETCH_FOLDER, FILE_HANDLER, GPODDER_INTEGRATION_ENABLED, OIDC_AUTH, OIDC_AUTHORITY,
    OIDC_CLIENT_ID, OIDC_JWKS, OIDC_REDIRECT_URI, OIDC_REFRESH_INTERVAL, OIDC_SCOPE, PASSWORD,
    PODFETCH_FOLDER, PODFETCH_PROXY_FOR_REQUESTS, PODINDEX_API_KEY, PODINDEX_API_SECRET,
    POLLING_INTERVAL, POLLING_INTERVAL_DEFAULT, REVERSE_PROXY, REVERSE_PROXY_AUTO_SIGN_UP,
    REVERSE_PROXY_HEADER, S3_ACCESS_KEY, S3_PROFILE, S3_REGION, S3_SECRET_KEY, S3_SECURITY_TOKEN,
    S3_SESSION_TOKEN, S3_URL, SERVER_URL, SUB_DIRECTORY, TELEGRAM_API_ENABLED,
    TELEGRAM_BOT_CHAT_ID, TELEGRAM_BOT_TOKEN, USERNAME,
};

use crate::models::episode::Episode;
use crate::models::podcast_episode::PodcastEpisode;
use crate::utils::error::ErrorSeverity::Warning;
use crate::utils::error::{CustomError, CustomErrorInner};
use podfetch_domain::favorite_podcast_episode::FavoritePodcastEpisode;
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

impl TryFrom<String> for Role {
    type Error = CustomError;

    fn try_from(value: String) -> Result<Self, CustomError> {
        match value.as_str() {
            "admin" => Ok(Role::Admin),
            "uploader" => Ok(Role::Uploader),
            "user" => Ok(Role::User),
            "Admin" => Ok(Role::Admin),
            "Uploader" => Ok(Role::Uploader),
            "User" => Ok(Role::User),
            _ => Err(CustomErrorInner::BadRequest("Invalid role".to_string(), Warning).into()),
        }
    }
}

impl Role {
    pub const VALUES: [Self; 3] = [Self::User, Self::Admin, Self::Uploader];
}

pub const STANDARD_USER: &str = "user123";
pub const STANDARD_USER_ID: i32 = 9999;

pub const PODCAST_FILENAME: &str = "podcast";
pub const PODCAST_IMAGENAME: &str = "image";

pub const CSS: &str = "css";
pub const JS: &str = "javascript";

pub const COMMON_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
(KHTML, like Gecko) Chrome/116.0.0.0 Safari/537.36";

// Default device when viewing via web interface
pub const DEFAULT_DEVICE: &str = "webview";

// static constants

pub static ENVIRONMENT_SERVICE: LazyLock<EnvironmentService> = LazyLock::new(|| {
    init_logging();
    #[cfg(test)]
    {
        let env = EnvironmentService::for_tests();
        println!("Environment: {:?}", env.database_url);
        env
    }
    #[cfg(not(test))]
    EnvironmentService::new()
});

pub static DEFAULT_IMAGE_URL: &str = "ui/default.jpg";

pub const MAIN_ROOM: &str = "main";

pub type PodcastEpisodeWithFavorited = Result<
    Vec<(
        PodcastEpisode,
        Option<Episode>,
        Option<FavoritePodcastEpisode>,
    )>,
    CustomError,
>;
