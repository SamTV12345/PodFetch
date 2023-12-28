use crate::config::dbconfig::get_database_url;
use crate::constants::inner_constants::{
    BASIC_AUTH, GPODDER_INTEGRATION_ENABLED, OIDC_AUTH, OIDC_AUTHORITY, OIDC_CLIENT_ID,
    OIDC_REDIRECT_URI, OIDC_SCOPE, PASSWORD, PODINDEX_API_KEY, PODINDEX_API_SECRET,
    POLLING_INTERVAL, POLLING_INTERVAL_DEFAULT, SERVER_URL, SUB_DIRECTORY, USERNAME,
};
use crate::models::settings::ConfigModel;
use crate::utils::environment_variables::is_env_var_present_and_true;
use regex::Regex;
use std::env;
use std::env::var;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OidcConfig {
    authority: String,
    client_id: String,
    redirect_uri: String,
    scope: String,
}

#[derive(Clone)]
pub struct EnvironmentService {
    pub server_url: String,
    pub polling_interval: u32,
    pub podindex_api_key: String,
    pub podindex_api_secret: String,
    pub http_basic: bool,
    pub username: String,
    pub password: String,
    pub oidc_config: Option<OidcConfig>,
    pub oidc_configured: bool,
    pub gpodder_integration_enabled: bool,
}

impl Default for EnvironmentService {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvironmentService {
    pub fn new() -> EnvironmentService {
        let mut option_oidc_config = None;
        let oidc_configured = is_env_var_present_and_true(OIDC_AUTH);
        if oidc_configured {
            option_oidc_config = Some(OidcConfig {
                redirect_uri: var(OIDC_REDIRECT_URI).expect("OIDC redirect uri not configured"),
                authority: var(OIDC_AUTHORITY).expect("OIDC authority not configured"),
                client_id: var(OIDC_CLIENT_ID).expect("OIDC client id not configured"),
                scope: var(OIDC_SCOPE).unwrap_or("openid profile email".to_string()),
            });
        }
        let mut server_url = var(SERVER_URL).unwrap_or("http://localhost:8000".to_string());
        // Add trailing slash if not present
        if !server_url.ends_with('/') {
            server_url += "/"
        }

        if var(SUB_DIRECTORY).is_err() {
            let re = Regex::new(r"^http[s]?://[^/]+(/.*)?$").unwrap();
            let directory = re.captures(&server_url).unwrap().get(1).unwrap().as_str();
            if directory.ends_with('/') {
                env::set_var(SUB_DIRECTORY, &directory[0..directory.len() - 1]);
            } else {
                env::set_var(SUB_DIRECTORY, directory);
            }
        }

        EnvironmentService {
            server_url: server_url.clone(),
            polling_interval: var(POLLING_INTERVAL)
                .unwrap_or(POLLING_INTERVAL_DEFAULT.to_string())
                .parse::<u32>()
                .unwrap(),
            podindex_api_key: var(PODINDEX_API_KEY).unwrap_or("".to_string()),
            podindex_api_secret: var(PODINDEX_API_SECRET).unwrap_or("".to_string()),
            http_basic: is_env_var_present_and_true(BASIC_AUTH),
            username: var(USERNAME).unwrap_or("".to_string()),
            password: var(PASSWORD).unwrap_or("".to_string()),
            oidc_configured,
            oidc_config: option_oidc_config,
            gpodder_integration_enabled: is_env_var_present_and_true(GPODDER_INTEGRATION_ENABLED),
        }
    }

    pub fn get_server_url(&self) -> String {
        self.server_url.clone()
    }

    pub fn get_podindex_api_key(&self) -> String {
        self.podindex_api_key.clone()
    }

    pub fn get_podindex_api_secret(&self) -> String {
        self.podindex_api_secret.clone()
    }

    pub fn get_polling_interval(&self) -> u32 {
        self.polling_interval
    }

    pub fn get_environment(&self) {
        println!("\n");
        log::info!("Starting with the following environment variables:");
        for (key, value) in env::vars() {
            log::debug!("{}: {}", key, value);
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
        log::debug!("Database url is set to: {}", &get_database_url());
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
            basic_auth: self.http_basic,
            oidc_configured: self.oidc_configured,
            oidc_config: self.oidc_config.clone(),
        }
    }

    pub fn get_api_key(&self) {}

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
    use crate::constants::inner_constants::{BASIC_AUTH, ENVIRONMENT_SERVICE, OIDC_AUTH, OIDC_AUTHORITY, OIDC_CLIENT_ID, OIDC_REDIRECT_URI, OIDC_SCOPE, PASSWORD, PODINDEX_API_KEY, PODINDEX_API_SECRET, POLLING_INTERVAL, SERVER_URL, USERNAME};
    
    use serial_test::serial;
    use std::env::{remove_var, set_var};

    fn do_env_cleanup() {
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

    #[test]
    #[serial]
    fn test_get_config() {
        do_env_cleanup();

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
        let env_service = ENVIRONMENT_SERVICE.get().unwrap();
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
        set_var(SERVER_URL, "http://localhost:8000");

        assert_eq!(ENVIRONMENT_SERVICE.get().unwrap().get_server_url(), "http://localhost:8000/");
    }

    #[test]
    #[serial]
    fn test_get_config_without_oidc() {
        do_env_cleanup();
        set_var(SERVER_URL, "http://localhost:8000");
        set_var(PODINDEX_API_KEY, "test");
        set_var(PODINDEX_API_SECRET, "test");
        set_var(POLLING_INTERVAL, "10");
        set_var(BASIC_AUTH, "true");
        set_var(USERNAME, "test");
        set_var(PASSWORD, "test");
        let config = ENVIRONMENT_SERVICE.get().unwrap().get_config();
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
        set_var(PODINDEX_API_KEY, "test");
        set_var(PODINDEX_API_SECRET, "testsecret");

        assert_eq!(ENVIRONMENT_SERVICE.get().unwrap().get_podindex_api_key(), "test");
        assert_eq!(ENVIRONMENT_SERVICE.get().unwrap().get_podindex_api_secret(), "testsecret");
    }

    #[test]
    #[serial]
    fn test_get_polling_interval() {
        do_env_cleanup();
        set_var(POLLING_INTERVAL, "20");
        assert_eq!(ENVIRONMENT_SERVICE.get().unwrap().get_polling_interval(), 20);
    }
}
