use serde_json::Value;
use crate::service::environment_service::EnvironmentService;

pub fn unwrap_string(value: &Value) -> String {
    return value.to_string().replace("\"", "");
}

pub fn unwrap_string_audio(value: &Value) -> String {
    return match value.to_string().is_empty() {
        true => {
            let env = EnvironmentService::new();
            let url = env.server_url.clone().to_owned() + &"ui/default.jpg".to_string();
            url
        },
        false => {
            value.to_string().replace("\"", "")
        }
    }
}

pub fn get_default_image() -> String {
    let env = EnvironmentService::new();
    let url = env.server_url.clone().to_owned() + &"ui/default.jpg".to_string();
    url
}
