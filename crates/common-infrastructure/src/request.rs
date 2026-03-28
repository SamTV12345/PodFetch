use base64::Engine;
use base64::engine::general_purpose;
use regex::Regex;
use std::sync::LazyLock;

static BASIC_AUTH_COND_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r".*//([^/?#\s]+)@.*").expect("valid basic auth regex"));

pub fn add_basic_auth_headers_conditionally(
    url: String,
    header_map: &mut reqwest::header::HeaderMap,
) {
    if url.contains('@')
        && let Some(captures) = BASIC_AUTH_COND_REGEX.captures(&url)
        && let Some(auth) = captures.get(1)
    {
        let b64_auth = general_purpose::STANDARD.encode(auth.as_str());
        let mut bearer = String::from("Basic ");
        bearer.push_str(&b64_auth);
        header_map.append("Authorization", bearer.parse().unwrap());
    }
}

#[cfg(test)]
mod tests {
    use super::add_basic_auth_headers_conditionally;

    #[test]
    fn check_if_adding_headers_works_with_at_sign() {
        let mut header_map = reqwest::header::HeaderMap::new();
        add_basic_auth_headers_conditionally(
            "https://user:pass@localhost:8080".to_string(),
            &mut header_map,
        );

        assert_eq!(header_map.len(), 1);
        assert_eq!(
            header_map.get("Authorization").unwrap().to_str().unwrap(),
            "Basic dXNlcjpwYXNz"
        );
    }

    #[test]
    fn check_if_adding_headers_works_with_at_sign_complicated_password() {
        let mut header_map = reqwest::header::HeaderMap::new();
        add_basic_auth_headers_conditionally(
            "https://user123123:Jm7YAT8m5YA8Forx7w6wsmUXDvcJny@localhost:8080".to_string(),
            &mut header_map,
        );

        assert_eq!(header_map.len(), 1);
        assert_eq!(
            header_map.get("Authorization").unwrap().to_str().unwrap(),
            "Basic dXNlcjEyMzEyMzpKbTdZQVQ4bTVZQThGb3J4N3c2d3NtVVhEdmNKbnk="
        );
    }

    #[test]
    fn check_if_adding_headers_works_with_at_sign_and_existing_header() {
        let mut header_map = reqwest::header::HeaderMap::new();
        header_map.append("Content-Type", "application/json".parse().unwrap());

        add_basic_auth_headers_conditionally(
            "https://user:pass@localhost:8080".to_string(),
            &mut header_map,
        );

        assert_eq!(header_map.len(), 2);
    }
}
