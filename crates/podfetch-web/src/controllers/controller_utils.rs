use common_infrastructure::runtime::DEFAULT_IMAGE_URL;
use serde_json::Value;

use crate::url_rewriting::normalize_server_url;

pub fn unwrap_string(value: &Value) -> String {
    value.to_string().replace('\"', "")
}

pub fn unwrap_string_audio(value: &Value, server_url: &str) -> String {
    if value.to_string().is_empty() {
        format!("{}{DEFAULT_IMAGE_URL}", normalize_server_url(server_url))
    } else {
        value.to_string().replace('\"', "")
    }
}

pub fn get_default_image(server_url: &str) -> String {
    format!("{}{DEFAULT_IMAGE_URL}", normalize_server_url(server_url))
}
