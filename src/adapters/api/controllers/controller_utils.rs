use serde_json::Value;
use crate::constants::inner_constants::{DEFAULT_IMAGE_URL, ENVIRONMENT_SERVICE};

pub fn unwrap_string(value: &Value) -> String {
    value.to_string().replace('\"', "")
}

pub fn unwrap_string_audio(value: &Value) -> String {
    match value.to_string().is_empty() {
        true => {
            let env = ENVIRONMENT_SERVICE.get().unwrap();

            env.server_url.clone().to_owned() + DEFAULT_IMAGE_URL
        }
        false => value.to_string().replace('\"', ""),
    }
}

pub fn get_default_image() -> String {
    let env = ENVIRONMENT_SERVICE.get().unwrap();

    env.server_url.clone().to_owned() + DEFAULT_IMAGE_URL
}
