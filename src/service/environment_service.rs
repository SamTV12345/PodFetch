use crate::adapters::file::file_handler::FileHandlerType;

#[cfg(feature = "postgresql")]
use crate::constants::inner_constants::CONNECTION_NUMBERS;

use crate::constants::inner_constants::{
    API_KEY, BASIC_AUTH, DATABASE_URL, DATABASE_URL_DEFAULT_SQLITE, DEFAULT_OIDC_REFRESH_INTERVAL,
    DEFAULT_PODFETCH_FOLDER, FILE_HANDLER, GPODDER_INTEGRATION_ENABLED, OIDC_AUTH, OIDC_AUTHORITY,
    OIDC_CLIENT_ID, OIDC_JWKS, OIDC_REDIRECT_URI, OIDC_REFRESH_INTERVAL, OIDC_SCOPE, PASSWORD,
    PODFETCH_FOLDER, PODFETCH_PROXY_FOR_REQUESTS, PODINDEX_API_KEY, PODINDEX_API_SECRET,
    POLLING_INTERVAL, POLLING_INTERVAL_DEFAULT, REVERSE_PROXY, REVERSE_PROXY_AUTO_SIGN_UP,
    REVERSE_PROXY_HEADER, S3_ACCESS_KEY, S3_PROFILE, S3_REGION, S3_SECRET_KEY, S3_SECURITY_TOKEN,
    S3_SESSION_TOKEN, S3_URL, SERVER_URL, SUB_DIRECTORY, TELEGRAM_API_ENABLED,
    TELEGRAM_BOT_CHAT_ID, TELEGRAM_BOT_TOKEN, USERNAME,
};
use crate::models::settings::ConfigModel;
use crate::utils::environment_variables::is_env_var_present_and_true;
use s3::creds::Credentials;
use s3::error::S3Error;
use s3::{Bucket, Region};
use std::env;
use std::env::var;
use url::Url;
use utoipa::ToSchema;

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

impl EnvironmentService {
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
        // Safe to unwrap as the server url is validated on startup
        url::Url::parse(&rss_feed_url).unwrap()
    }

    pub fn new() -> EnvironmentService {
        let oidc_configured = Self::handle_oidc();

        let server_url = match var("DEV") {
            Ok(_) => "http://localhost:5173/".to_string(),
            Err(_) => var(SERVER_URL)
                .map(|s| if s.ends_with('/') { s } else { s + "/" })
                .unwrap_or("http://localhost:8000/".to_string()),
        };

        let ws_url = match var("DEV") {
            Ok(_) => "http://localhost:8000/socket.io/".to_string(),
            Err(_) => var(SERVER_URL)
                .map(|mut s| {
                    s = match s.starts_with("https") {
                        true => s.replace("https", "wss"),
                        false => s.replace("http", "ws"),
                    };
                    if s.ends_with('/') {
                        s + "socket.io"
                    } else {
                        s + "/socket.io"
                    }
                })
                .unwrap_or("http://localhost:8000/socket.io/".to_string()),
        };

        let mut opt_sub_dir = var(SUB_DIRECTORY)
            .map_err(|_| None::<String>)
            .map(Some)
            .unwrap_or(None);
        if opt_sub_dir.is_none() {
            let url = Url::parse(&server_url).expect("Invalid server url");
            if url.path().ends_with('/') {
                opt_sub_dir = Some(url.path()[0..url.path().len() - 1].to_string());
            } else {
                opt_sub_dir = Some(url.path().to_string());
            }
        }

        let username_send: Option<String>;

        if let Ok(username) = var(USERNAME) {
            username_send = Some(username);
        } else {
            username_send = None;
        }

        let password: Option<String>;

        if let Ok(password_present) = var(PASSWORD) {
            let digested_password = sha256::digest(password_present);
            password = Some(digested_password)
        } else {
            password = None;
        }

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
                .map(|v| {
                    v.parse::<u32>()
                        .map_err(|_| POLLING_INTERVAL_DEFAULT)
                        .unwrap_or(POLLING_INTERVAL_DEFAULT)
                })
                .unwrap_or(POLLING_INTERVAL_DEFAULT),
            podindex_api_key: var(PODINDEX_API_KEY).unwrap_or("".to_string()),
            podindex_api_secret: var(PODINDEX_API_SECRET).unwrap_or("".to_string()),
            http_basic: is_env_var_present_and_true(BASIC_AUTH),
            username: username_send,
            password,
            oidc_configured: oidc_configured.is_some(),
            oidc_config: oidc_configured,
            reverse_proxy_config,
            gpodder_integration_enabled: is_env_var_present_and_true(GPODDER_INTEGRATION_ENABLED),
            database_url: var(DATABASE_URL).unwrap_or(DATABASE_URL_DEFAULT_SQLITE.to_string()),
            telegram_api,
            sub_directory: opt_sub_dir,
            proxy_url: var(PODFETCH_PROXY_FOR_REQUESTS).map(Some).unwrap_or(None),
            reverse_proxy: is_env_var_present_and_true(REVERSE_PROXY),
            any_auth_enabled: is_env_var_present_and_true(BASIC_AUTH)
                || is_env_var_present_and_true(OIDC_AUTH)
                || is_env_var_present_and_true(REVERSE_PROXY),
            #[cfg(feature = "postgresql")]
            conn_number: var(CONNECTION_NUMBERS)
                .unwrap_or("10".to_string())
                .parse::<i16>()
                .unwrap_or(10),
            api_key_admin: var(API_KEY).map(Some).unwrap_or(None),
            default_file_handler: handler.0,
            s3_config: handler.1,
            default_podfetch_folder: var(PODFETCH_FOLDER)
                .unwrap_or(DEFAULT_PODFETCH_FOLDER.to_string()),
        }
    }

    fn handle_default_file_handler() -> (FileHandlerType, S3Config) {
        match var(FILE_HANDLER) {
            Ok(handler) => match handler.as_str() {
                "s3" => {
                    log::info!("Using S3 file handler");
                    (FileHandlerType::S3, Self::capture_s3_config())
                }
                _ => {
                    log::info!("Using local file handler");
                    (FileHandlerType::Local, Self::capture_s3_config())
                }
            },
            Err(_) => (FileHandlerType::Local, Self::capture_s3_config()),
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
            let telegram_bot_token = var(TELEGRAM_BOT_TOKEN).ok().map_or_else(
                || {
                    log::error!("Telegram bot token not configured");
                    std::process::exit(1);
                },
                |v| v,
            );

            let telegram_chat_id = var(TELEGRAM_BOT_CHAT_ID).ok().map_or_else(
                || {
                    log::error!("Telegram chat id not configured");
                    std::process::exit(1);
                },
                |v| v,
            );

            Some(TelegramConfig {
                telegram_bot_token,
                telegram_chat_id,
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
            self.server_url.clone()
                + "swagger-ui/index\
        .html#/"
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

#[cfg(test)]
mod tests {
    use crate::constants::inner_constants::{
        BASIC_AUTH, OIDC_AUTH, OIDC_AUTHORITY, OIDC_CLIENT_ID, OIDC_JWKS, OIDC_REDIRECT_URI,
        OIDC_SCOPE, PASSWORD, PODINDEX_API_KEY, PODINDEX_API_SECRET, POLLING_INTERVAL, SERVER_URL,
        USERNAME,
    };

    use crate::service::environment_service::EnvironmentService;
    use serial_test::serial;
    use std::env::{remove_var, set_var};

    fn do_env_cleanup() {
        unsafe {
            remove_var(SERVER_URL);
            remove_var(PODINDEX_API_KEY);
            remove_var(PODINDEX_API_SECRET);
            remove_var(POLLING_INTERVAL);
            remove_var(BASIC_AUTH);
            remove_var(USERNAME);
            remove_var(PASSWORD);
            remove_var(OIDC_AUTH);
            remove_var(OIDC_REDIRECT_URI);
            remove_var(OIDC_AUTHORITY);
            remove_var(OIDC_CLIENT_ID);
            remove_var(OIDC_SCOPE);
        }
    }

    #[test]
    #[serial]
    fn test_get_config() {
        do_env_cleanup();

        unsafe {
            set_var(SERVER_URL, "http://localhost:8000");
            set_var(POLLING_INTERVAL, "10");
            set_var(BASIC_AUTH, "true");
            set_var(USERNAME, "test");
            set_var(PASSWORD, "test");
            set_var(OIDC_AUTH, "true");
            set_var(OIDC_REDIRECT_URI, "http://localhost:8000/oidc");
            set_var(OIDC_AUTHORITY, "http://localhost:8000/oidc");
            set_var(OIDC_CLIENT_ID, "test");
            set_var(OIDC_SCOPE, "openid profile email");
            set_var(OIDC_JWKS, "test");
        }

        let env_service = EnvironmentService::new();
        let config = env_service.get_config();
        assert!(!config.podindex_configured);
        assert_eq!(config.rss_feed, "http://localhost:8000/rss");
        assert_eq!(config.server_url, "http://localhost:8000/");
        assert!(config.basic_auth);
        assert!(config.oidc_configured);
        assert_eq!(
            config.oidc_config.clone().unwrap().clone().client_id,
            "test"
        );
        assert_eq!(
            config.oidc_config.clone().unwrap().clone().redirect_uri,
            "http://localhost:8000/oidc"
        );
        assert_eq!(
            config.oidc_config.clone().unwrap().clone().scope,
            "openid profile email"
        );
    }

    #[test]
    #[serial]
    fn test_getting_server_url() {
        do_env_cleanup();
        unsafe {
            set_var(SERVER_URL, "http://localhost:8000");
        }

        let env_service = EnvironmentService::new();
        assert_eq!(env_service.get_server_url(), "http://localhost:8000/");
    }

    #[test]
    #[serial]
    fn test_get_config_without_oidc() {
        do_env_cleanup();
        unsafe {
            set_var(SERVER_URL, "http://localhost:8000");
            set_var(PODINDEX_API_KEY, "test");
            set_var(PODINDEX_API_SECRET, "test");
            set_var(POLLING_INTERVAL, "10");
            set_var(BASIC_AUTH, "true");
            set_var(USERNAME, "test");
            set_var(PASSWORD, "test");
        }

        let config = EnvironmentService::new().get_config();
        assert!(config.podindex_configured);
        assert_eq!(config.rss_feed, "http://localhost:8000/rss");
        assert_eq!(config.server_url, "http://localhost:8000/");
        assert!(config.basic_auth);
        assert!(!config.oidc_configured);
    }

    #[test]
    #[serial]
    fn test_get_podindex_api_key() {
        do_env_cleanup();
        unsafe {
            set_var(PODINDEX_API_KEY, "test");
            set_var(PODINDEX_API_SECRET, "testsecret");
        }
        let env_service = EnvironmentService::new();
        assert_eq!(env_service.podindex_api_key, "test");
        assert_eq!(env_service.podindex_api_secret, "testsecret");
    }

    #[test]
    #[serial]
    fn test_get_polling_interval() {
        do_env_cleanup();
        unsafe {
            set_var(POLLING_INTERVAL, "20");
        }
        assert_eq!(EnvironmentService::new().polling_interval, 20);
    }
}
