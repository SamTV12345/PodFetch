use std::env::{var};
use crate::models::settings::ConfigModel;

#[derive(Clone)]
pub struct EnvironmentService {
    pub server_url: String,
    pub polling_interval: u32,
    pub podindex_api_key: String,
    pub podindex_api_secret: String,
}

impl EnvironmentService {
    pub fn new() -> EnvironmentService {
        EnvironmentService {
            server_url: var("SERVER_URL").unwrap_or("http://localhost:8000".to_string()),
            polling_interval: var("POLLING_INTERVAL").unwrap_or("300".to_string()).parse::<u32>()
                .unwrap(),
            podindex_api_key: var("PODINDEX_API_KEY").unwrap_or("".to_string()),
            podindex_api_secret: var("PODINDEX_API_SECRET").unwrap_or("".to_string())
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

    pub fn get_environment(&self){
        log::info!("Starting with the following environment variables:");
        for (key, value) in std::env::vars() {
            log::debug!("{}: {}", key, value);
        }
        println!("Public server url: {}", self.server_url);
        println!("Polling interval for new episodes: {} minutes", self.polling_interval);
        println!("Podindex API key&secret configured: {}", self.podindex_api_key.len()>0&& self.podindex_api_secret.len()>0);
    }

    pub fn get_config(&mut self)->ConfigModel{
        ConfigModel{
            podindex_configured: self.podindex_api_key.len()>0&& self.podindex_api_secret.len()>0
        }
    }

    pub fn get_api_key(&self){

    }


    pub fn print_banner(){
        println!(r"
.______     ______    _______   _______ .______          ___      .______      ____    ____  ___
|   _  \   /  __  \  |       \ /  _____||   _  \        /   \     |   _  \     \   \  /   / |__ \
|  |_)  | |  |  |  | |  .--.  |  |  __  |  |_)  |      /  ^  \    |  |_)  |     \   \/   /     ) |
|   ___/  |  |  |  | |  |  |  |  | |_ | |      /      /  /_\  \   |   _  <       \      /     / /
|  |      |  `--'  | |  '--'  |  |__| | |  |\  \----./  _____  \  |  |_)  |       \    /     / /_
| _|       \______/  |_______/ \______| | _| `._____/__/     \__\ |______/         \__/     |____|

        ")
    }
}
