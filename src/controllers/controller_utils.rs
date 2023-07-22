use serde_json::Value;
use crate::service::environment_service::EnvironmentService;

pub fn unwrap_string(value: &Value) -> String {
    value.to_string().replace('\"', "")
}

pub fn unwrap_string_audio(value: &Value) -> String {
    match value.to_string().is_empty() {
        true => {
            let env = EnvironmentService::new();
            
            env.server_url.clone().to_owned() + "ui/default.jpg"
        },
        false => {
            value.to_string().replace('\"', "")
        }
    }
}

pub fn get_default_image() -> String {
    let env = EnvironmentService::new();
    
    env.server_url.clone().to_owned() + "ui/default.jpg"
}
