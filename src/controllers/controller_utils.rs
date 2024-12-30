use crate::constants::inner_constants::{DEFAULT_IMAGE_URL, ENVIRONMENT_SERVICE};
use serde_json::Value;

pub fn unwrap_string(value: &Value) -> String {
    value.to_string().replace('\"', "")
}

pub fn unwrap_string_audio(value: &Value) -> String {
    match value.to_string().is_empty() {
        true => ENVIRONMENT_SERVICE.server_url.clone().to_owned() + DEFAULT_IMAGE_URL,
        false => value.to_string().replace('\"', ""),
    }
}

pub fn get_default_image() -> String {
    ENVIRONMENT_SERVICE.server_url.clone().to_owned() + DEFAULT_IMAGE_URL
}
