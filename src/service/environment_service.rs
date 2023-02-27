use std::env::{var};


pub struct EnvironmentService {
    pub server_url: String,
}

impl EnvironmentService {
    pub fn new() -> EnvironmentService {
        EnvironmentService {
            server_url: var("SERVER_URL").unwrap_or("http://localhost:8000".to_string()),
        }
    }

    pub fn get_server_url(&self) -> String {
        self.server_url.clone()
    }

    pub fn get_polling_interval(&self) -> u32 {
        var("POLLING_INTERVAL").unwrap_or("300".to_string()).parse::<u32>().unwrap()
    }
}