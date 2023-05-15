use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

pub static ITUNES_URL: &str = "https://itunes.apple.com/search?term=";

#[derive(Serialize, Deserialize, Debug)]
pub enum PodcastType {
    AddPodcast,
    AddPodcastEpisode,
    AddPodcastEpisodes,
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

pub const ERROR_LOGIN_MESSAGE: &str = "User either not found or password is incorrect";

pub const TELEGRAM_BOT_TOKEN: &str = "TELEGRAM_BOT_TOKEN";
pub const TELEGRAM_BOT_CHAT_ID: &str = "TELEGRAM_BOT_CHAT_ID";
pub const TELEGRAM_API_ENABLED: &str = "TELEGRAM_API_ENABLED";


// User management roles
#[derive( Serialize, Deserialize, Debug, PartialEq)]
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
pub const BASIC_AUTH:&str = "BASIC_AUTH";


pub const USERNAME:&str = "USERNAME";
pub const PASSWORD:&str = "PASSWORD";


pub const STANDARD_USER: &str = "user123";


pub const ERR_SETTINGS_FORMAT: &str = "A podcast/episode format needs to contain an opening and \
closing bracket ({}).";