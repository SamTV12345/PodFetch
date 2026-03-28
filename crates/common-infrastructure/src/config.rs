use s3::creds::Credentials;
use s3::error::S3Error;
use s3::{Bucket, Region};
use serde::{Deserialize, Serialize};
use std::env;
use std::env::var;
use std::fmt::{Display, Formatter};
use url::Url;
use utoipa::ToSchema;

pub const TELEGRAM_BOT_TOKEN: &str = "TELEGRAM_BOT_TOKEN";
pub const TELEGRAM_BOT_CHAT_ID: &str = "TELEGRAM_BOT_CHAT_ID";
pub const TELEGRAM_API_ENABLED: &str = "TELEGRAM_API_ENABLED";
pub const PODFETCH_FOLDER: &str = "PODFETCH_FOLDER";
pub const DEFAULT_PODFETCH_FOLDER: &str = "podcasts";
pub const FILE_HANDLER: &str = "FILE_HANDLER";
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
pub const PODINDEX_API_KEY: &str = "PODINDEX_API_KEY";
pub const PODINDEX_API_SECRET: &str = "PODINDEX_API_SECRET";
pub const GPODDER_INTEGRATION_ENABLED: &str = "GPODDER_INTEGRATION_ENABLED";
pub const DATABASE_URL: &str = "DATABASE_URL";
pub const DATABASE_URL_DEFAULT_SQLITE: &str = "sqlite://./podcast.db";
pub const OIDC_JWKS: &str = "OIDC_JWKS";
pub const OIDC_REFRESH_INTERVAL: &str = "OIDC_REFRESH_INTERVAL";
pub const DEFAULT_OIDC_REFRESH_INTERVAL: u64 = 1000 * 60 * 2;
pub const REVERSE_PROXY: &str = "REVERSE_PROXY";
pub const REVERSE_PROXY_HEADER: &str = "REVERSE_PROXY_HEADER";
pub const REVERSE_PROXY_AUTO_SIGN_UP: &str = "REVERSE_PROXY_AUTO_SIGN_UP";
pub const PODFETCH_PROXY_FOR_REQUESTS: &str = "PODFETCH_PROXY";
#[cfg(feature = "postgresql")]
pub const CONNECTION_NUMBERS: &str = "DB_CONNECTIONS";
pub const S3_URL: &str = "S3_URL";
pub const S3_REGION: &str = "S3_REGION";
pub const S3_ACCESS_KEY: &str = "S3_ACCESS_KEY";
pub const S3_SECRET_KEY: &str = "S3_SECRET_KEY";
pub const S3_PROFILE: &str = "S3_PROFILE";
pub const S3_SECURITY_TOKEN: &str = "S3_SECURITY_TOKEN";
pub const S3_SESSION_TOKEN: &str = "S3_SESSION_TOKEN";
pub const POLLING_INTERVAL_DEFAULT: u32 = 300;

pub fn is_env_var_present_and_true(env_var: &str) -> bool {
    match env::var(env_var) {
        Ok(val) => val == "true" || val == "1" || val == "yes",
        Err(_) => false,
    }
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum FileHandlerType {
    Local,
    S3,
}

impl Display for FileHandlerType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FileHandlerType::Local => write!(f, "Local"),
            FileHandlerType::S3 => write!(f, "S3"),
        }
    }
}

impl From<&str> for FileHandlerType {
    fn from(value: &str) -> Self {
        match value {
            "Local" => FileHandlerType::Local,
            "S3" => FileHandlerType::S3,
            _ => panic!("Invalid FileHandlerType"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OidcConfig {
    pub authority: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub scope: String,
    pub jwks_uri: String,
    pub refresh_interval: u64,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConfigModel {
    pub podindex_configured: bool,
    pub rss_feed: String,
    pub server_url: String,
    pub ws_url: String,
    pub basic_auth: bool,
    pub oidc_configured: bool,
    pub oidc_config: Option<OidcConfig>,
    pub reverse_proxy: bool,
}

#[derive(Clone)]
pub struct EnvironmentService {
    pub server_url: String,
    pub ws_url: String,
    pub polling_interval: u32,
    pub podindex_api_key: String,
    pub podindex_api_secret: String,
    pub http_basic: bool,
    pub username: Option<String>,
    pub password: Option<String>,
    pub oidc_config: Option<OidcConfig>,
    pub oidc_configured: bool,
    pub reverse_proxy: bool,
    pub reverse_proxy_config: Option<ReverseProxyConfig>,
    pub gpodder_integration_enabled: bool,
    pub database_url: String,
    pub telegram_api: Option<TelegramConfig>,
    pub any_auth_enabled: bool,
    pub sub_directory: Option<String>,
    pub proxy_url: Option<String>,
    #[cfg(feature = "postgresql")]
    pub conn_number: i16,
    pub api_key_admin: Option<String>,
    pub default_file_handler: FileHandlerType,
    pub default_podfetch_folder: String,
    pub s3_config: S3Config,
}

#[derive(Clone)]
pub struct S3Config {
    pub access_key: String,
    pub secret_key: String,
    pub security_token: Option<String>,
    pub session_token: Option<String>,
    pub profile: Option<String>,
    pub region: String,
    pub endpoint: String,
    pub bucket: String,
}

impl From<&S3Config> for Region {
    fn from(val: &S3Config) -> Self {
        Region::Custom {
            region: val.region.clone(),
            endpoint: val.endpoint.clone(),
        }
    }
}

impl S3Config {
    pub fn convert_to_string(self, id: &str) -> String {
        format!("/{}/{}", self.bucket, id)
    }
}

impl From<&S3Config> for Result<Box<Bucket>, S3Error> {
    fn from(val: &S3Config) -> Self {
        Bucket::new(val.bucket.as_str(), val.into(), val.into())
    }
}

impl From<&S3Config> for Credentials {
    fn from(val: &S3Config) -> Self {
        Credentials {
            access_key: Some(val.access_key.clone()),
            secret_key: Some(val.secret_key.clone()),
            security_token: val.security_token.clone(),
            session_token: val.session_token.clone(),
            expiration: None,
        }
    }
}

#[derive(Clone)]
pub struct ReverseProxyConfig {
    pub header_name: String,
    pub auto_sign_up: bool,
}

#[derive(Clone)]
pub struct TelegramConfig {
    pub telegram_bot_token: String,
    pub telegram_chat_id: String,
}

impl Default for EnvironmentService {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvironmentService {
    pub fn for_tests() -> Self {
        let mut environment = Self::new();
        #[cfg(feature = "sqlite")]
        {
            environment.database_url = "sqlite://./podcast.db".to_string();
        }
        #[cfg(feature = "postgresql")]
        {
            environment.database_url =
                "postgres://postgres:postgres@127.0.0.1:55002/postgres".to_string();
        }
        environment.http_basic = true;
        environment.username = Some("postgres".to_string());
        environment.password =
            Some("a942b37ccfaf5a813b1432caa209a43b9d144e47ad0de1549c289c253e556cd5".to_string());
        environment.gpodder_integration_enabled = true;
        environment
    }

    pub fn max_database_connections(&self) -> u32 {
        #[cfg(feature = "postgresql")]
        {
            self.conn_number as u32
        }
        #[cfg(not(feature = "postgresql"))]
        {
            16
        }
    }

    fn handle_oidc() -> Option<OidcConfig> {
        let oidc_configured = is_env_var_present_and_true(OIDC_AUTH);
        if oidc_configured {
            Some(OidcConfig {
                redirect_uri: var(OIDC_REDIRECT_URI).expect("OIDC redirect uri not configured"),
                authority: var(OIDC_AUTHORITY).expect("OIDC authority not configured"),
                client_id: var(OIDC_CLIENT_ID).expect("OIDC client id not configured"),
                scope: var(OIDC_SCOPE).unwrap_or("openid profile email".to_string()),
                jwks_uri: var(OIDC_JWKS).unwrap(),
                refresh_interval: var(OIDC_REFRESH_INTERVAL)
                    .unwrap_or(DEFAULT_OIDC_REFRESH_INTERVAL.to_string())
                    .parse::<u64>()
                    .unwrap_or(DEFAULT_OIDC_REFRESH_INTERVAL),
            })
        } else {
            None
        }
    }

    pub fn build_url_to_rss_feed(&self) -> Url {
        let mut rss_feed_url = self.server_url.to_string();
        rss_feed_url.push_str("rss");
        Url::parse(&rss_feed_url).unwrap()
    }

    pub fn new() -> EnvironmentService {
        let oidc_configured = Self::handle_oidc();

        let server_url = match var("DEV") {
            Ok(val) if val == "true" => "http://localhost:5173/".to_string(),
            _ => var(SERVER_URL)
                .map(|s| if s.ends_with('/') { s } else { s + "/" })
                .unwrap_or("http://localhost:8000/".to_string()),
        };

        let ws_url = match var("DEV") {
            Ok(_) => "http://localhost:8000/socket.io/".to_string(),
            Err(_) => var(SERVER_URL)
                .map(|mut s| {
                    s = if s.starts_with("https") {
                        s.replace("https", "wss")
                    } else {
                        s.replace("http", "ws")
                    };
                    if s.ends_with('/') {
                        s + "socket.io"
                    } else {
                        s + "/socket.io"
                    }
                })
                .unwrap_or("http://localhost:8000/socket.io/".to_string()),
        };

        let mut opt_sub_dir = var(SUB_DIRECTORY).ok();
        if opt_sub_dir.is_none() {
            let url = Url::parse(&server_url).expect("Invalid server url");
            if url.path().ends_with('/') {
                opt_sub_dir = Some(url.path()[0..url.path().len() - 1].to_string());
            } else {
                opt_sub_dir = Some(url.path().to_string());
            }
        }

        let password = var(PASSWORD).ok().map(sha256::digest);
        let telegram_api = Self::handle_telegram_config();
        let reverse_proxy_config = if is_env_var_present_and_true(REVERSE_PROXY) {
            Some(ReverseProxyConfig {
                header_name: var(REVERSE_PROXY_HEADER).unwrap_or("X-Forwarded-User".to_string()),
                auto_sign_up: is_env_var_present_and_true(REVERSE_PROXY_AUTO_SIGN_UP),
            })
        } else {
            None
        };
        let handler = Self::handle_default_file_handler();

        EnvironmentService {
            server_url: server_url.clone(),
            ws_url,
            polling_interval: var(POLLING_INTERVAL)
                .ok()
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(POLLING_INTERVAL_DEFAULT),
            podindex_api_key: var(PODINDEX_API_KEY).unwrap_or_default(),
            podindex_api_secret: var(PODINDEX_API_SECRET).unwrap_or_default(),
            http_basic: is_env_var_present_and_true(BASIC_AUTH),
            username: var(USERNAME).ok(),
            password,
            oidc_configured: oidc_configured.is_some(),
            oidc_config: oidc_configured,
            reverse_proxy_config,
            gpodder_integration_enabled: is_env_var_present_and_true(GPODDER_INTEGRATION_ENABLED),
            database_url: var(DATABASE_URL).unwrap_or(DATABASE_URL_DEFAULT_SQLITE.to_string()),
            telegram_api,
            sub_directory: opt_sub_dir,
            proxy_url: var(PODFETCH_PROXY_FOR_REQUESTS).ok(),
            reverse_proxy: is_env_var_present_and_true(REVERSE_PROXY),
            any_auth_enabled: is_env_var_present_and_true(BASIC_AUTH)
                || is_env_var_present_and_true(OIDC_AUTH)
                || is_env_var_present_and_true(REVERSE_PROXY),
            #[cfg(feature = "postgresql")]
            conn_number: var(CONNECTION_NUMBERS)
                .unwrap_or("10".to_string())
                .parse::<i16>()
                .unwrap_or(10),
            api_key_admin: var(API_KEY).ok(),
            default_file_handler: handler.0,
            s3_config: handler.1,
            default_podfetch_folder: var(PODFETCH_FOLDER)
                .unwrap_or(DEFAULT_PODFETCH_FOLDER.to_string()),
        }
    }

    fn handle_default_file_handler() -> (FileHandlerType, S3Config) {
        match var(FILE_HANDLER) {
            Ok(handler) if handler == "s3" => (FileHandlerType::S3, Self::capture_s3_config()),
            _ => (FileHandlerType::Local, Self::capture_s3_config()),
        }
    }

    fn capture_s3_config() -> S3Config {
        let mut endpoint = Self::variable_or_default(S3_URL, "http://localhost:9000");
        if endpoint.ends_with('/') {
            endpoint = endpoint[0..endpoint.len() - 1].to_string();
        }

        S3Config {
            region: Self::variable_or_default(S3_REGION, "eu-central-1"),
            access_key: Self::variable_or_default(S3_ACCESS_KEY, ""),
            secret_key: Self::variable_or_default(S3_SECRET_KEY, ""),
            endpoint,
            profile: Self::variable_or_option(S3_PROFILE),
            bucket: Self::variable_or_default(PODFETCH_FOLDER, "podcasts"),
            security_token: Self::variable_or_option(S3_SECURITY_TOKEN),
            session_token: Self::variable_or_option(S3_SESSION_TOKEN),
        }
    }

    fn variable_or_default(var_name: &str, default: &str) -> String {
        var(var_name).unwrap_or(default.to_string())
    }

    fn variable_or_option(var_name: &str) -> Option<String> {
        var(var_name).ok()
    }

    fn handle_telegram_config() -> Option<TelegramConfig> {
        if is_env_var_present_and_true(TELEGRAM_API_ENABLED) {
            Some(TelegramConfig {
                telegram_bot_token: var(TELEGRAM_BOT_TOKEN)
                    .unwrap_or_else(|_| panic!("Telegram bot token not configured")),
                telegram_chat_id: var(TELEGRAM_BOT_CHAT_ID)
                    .unwrap_or_else(|_| panic!("Telegram chat id not configured")),
            })
        } else {
            None
        }
    }

    pub fn get_server_url(&self) -> String {
        self.server_url.clone()
    }

    pub fn get_polling_interval(&self) -> u32 {
        self.polling_interval
    }

    pub fn get_environment(&self) {
        println!("\n");
        log::info!("Starting with the following environment variables:");
        for (key, value) in env::vars() {
            log::debug!("{key}: {value}");
        }
        log::info!("Public server url: {}", self.server_url);
        log::info!(
            "Polling interval for new episodes: {} minutes",
            self.polling_interval
        );
        log::info!(
            "Developer specifications available at {}",
            self.server_url.clone() + "swagger-ui/index.html#/"
        );
        log::info!(
            "GPodder integration enabled: {}",
            self.gpodder_integration_enabled
        );
        log::debug!("Database url is set to: {}", &self.database_url);
        log::info!(
            "Podindex API key&secret configured: {}",
            !self.podindex_api_key.is_empty() && !self.podindex_api_secret.is_empty()
        );
        println!("\n");
    }

    pub fn get_config(&self) -> ConfigModel {
        ConfigModel {
            podindex_configured: !self.podindex_api_key.is_empty()
                && !self.podindex_api_secret.is_empty(),
            rss_feed: self.server_url.clone() + "rss",
            server_url: self.server_url.clone(),
            reverse_proxy: self.reverse_proxy,
            basic_auth: self.http_basic,
            oidc_configured: self.oidc_configured,
            oidc_config: self.oidc_config.clone(),
            ws_url: self.ws_url.clone(),
        }
    }

    pub fn print_banner() {
        println!(
            r"
  ____           _ _____    _       _
 |  _ \ ___   __| |  ___|__| |_ ___| |__
 | |_) / _ \ / _` | |_ / _ \ __/ __| '_ \
 |  __/ (_) | (_| |  _|  __/ || (__| | | |
 |_|   \___/ \__,_|_|  \___|\__\___|_| |_|

        "
        )
    }
}
