use base64::Engine;
use base64::engine::general_purpose;
use regex::Regex;
use crate::models::itunes_models::PodcastEpisode;

pub fn add_basic_auth_headers_conditionally(url: String, header_map: &mut
reqwest::header::HeaderMap) {
    if url.contains("@") {
        let re = Regex::new(r"^(?:[^:/?#\s]+:)?//([^/?#\s]+@)?[^/?#\s]+").unwrap();
        if let Some(captures) = re.captures(&url) {
            if let Some(auth) = captures.get(1) {
                let b64_auth = general_purpose::STANDARD.encode(auth.as_str());
                header_map.append("Authorization", ("Basic ".to_owned() + &b64_auth).parse().unwrap());
            }
        }
    }
}