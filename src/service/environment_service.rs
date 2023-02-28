use std::env::{var};


pub struct EnvironmentService {
    pub server_url: String,
    pub polling_interval: u32,
}

impl EnvironmentService {
    pub fn new() -> EnvironmentService {
        EnvironmentService {
            server_url: var("SERVER_URL").unwrap_or("http://localhost:8000".to_string()),
            polling_interval: var("POLLING_INTERVAL").unwrap_or("300".to_string()).parse::<u32>()
                .unwrap()
        }
    }

    pub fn get_server_url(&self) -> String {
        self.server_url.clone()
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