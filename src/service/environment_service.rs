use std::env;
use crate::models::settings::ConfigModel;
use std::env::var;
use regex::Regex;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OidcConfig{
    authority: String,
    client_id: String,
    redirect_uri: String,
    scope: String
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
}

impl EnvironmentService {
    pub fn new() -> EnvironmentService {
        let mut option_oidc_config = None;
        let oidc_configured = var("OIDC_AUTH").is_ok();
        if oidc_configured{
            option_oidc_config = Some(OidcConfig{
                redirect_uri: var("OIDC_REDIRECT_URI").expect("OIDC redirect uri not configured"),
                authority: var("OIDC_AUTHORITY").expect("OIDC authority not configured"),
                client_id: var("OIDC_CLIENT_ID").expect("OIDC client id not configured"),
                scope: var("OIDC_SCOPE").unwrap_or("openid profile email".to_string())
            });
        }
        let mut server_url = var("SERVER_URL").unwrap_or("http://localhost:8000".to_string());
        // Add trailing slash if not present
        if !server_url.ends_with("/") {
            server_url+= "/"
        }

        if  !var("SUB_DIRECTORY").is_ok(){
            let re = Regex::new(r"^http[s]?://[^/]+(/.*)?$").unwrap();
            let directory = re.captures(&*server_url).unwrap().get(1).unwrap().as_str();
            if directory.ends_with("/"){
                env::set_var("SUB_DIRECTORY", &directory[0..directory.len()-1]);
            }
            else{
                env::set_var("SUB_DIRECTORY", directory);
            }
        }

        EnvironmentService {
            server_url: server_url.clone(),
            polling_interval: var("POLLING_INTERVAL")
                .unwrap_or("300".to_string())
                .parse::<u32>()
                .unwrap(),
            podindex_api_key: var("PODINDEX_API_KEY").unwrap_or("".to_string()),
            podindex_api_secret: var("PODINDEX_API_SECRET").unwrap_or("".to_string()),
            http_basic: var("BASIC_AUTH").is_ok(),
            username: var("USERNAME").unwrap_or("".to_string()),
            password: var("PASSWORD").unwrap_or("".to_string()),
            oidc_configured,
            oidc_config: option_oidc_config
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
        self.polling_interval.clone()
    }

    pub fn get_environment(&self) {
        log::info!("Starting with the following environment variables:");
        for (key, value) in env::vars() {
            log::debug!("{}: {}", key, value);
        }
        println!("Public server url: {}", self.server_url);
        println!(
            "Polling interval for new episodes: {} minutes",
            self.polling_interval
        );
        println!(
            "Podindex API key&secret configured: {}",
            self.podindex_api_key.len() > 0 && self.podindex_api_secret.len() > 0
        );
    }

    pub fn get_config(&mut self) -> ConfigModel {
        ConfigModel {
            podindex_configured: self.podindex_api_key.len() > 0
                && self.podindex_api_secret.len() > 0,
            rss_feed: self.server_url.clone() + "rss",
            server_url: self.server_url.clone(),
            basic_auth: self.http_basic,
            oidc_configured: self.oidc_configured,
            oidc_config: self.oidc_config.clone()
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
