use common_infrastructure::runtime::DEFAULT_IMAGE_URL;
use serde_json::Value;

use crate::url_rewriting::normalize_server_url;

pub fn unwrap_string(value: &Value) -> String {
    value.to_string().replace('\"', "")
}

pub fn unwrap_string_audio(value: &Value, server_url: &str) -> String {
    match value.to_string().is_empty() {
        true => format!("{}{DEFAULT_IMAGE_URL}", normalize_server_url(server_url)),
        false => value.to_string().replace('\"', ""),
    }
}

pub fn get_default_image(server_url: &str) -> String {
    format!("{}{DEFAULT_IMAGE_URL}", normalize_server_url(server_url))
}
