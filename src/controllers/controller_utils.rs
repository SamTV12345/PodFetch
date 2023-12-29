use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use serde_json::Value;

pub fn unwrap_string(value: &Value) -> String {
    value.to_string().replace('\"', "")
}

pub fn unwrap_string_audio(value: &Value) -> String {
    match value.to_string().is_empty() {
        true => {
            let env = ENVIRONMENT_SERVICE.get().unwrap();

            env.server_url.clone().to_owned() + "ui/default.jpg"
        }
        false => value.to_string().replace('\"', ""),
    }
}

pub fn get_default_image() -> String {
    let env = ENVIRONMENT_SERVICE.get().unwrap();

    env.server_url.clone().to_owned() + "ui/default.jpg"
}
