use base64::engine::general_purpose;
use base64::Engine;
use regex::Regex;

pub fn add_basic_auth_headers_conditionally(
    url: String,
    header_map: &mut reqwest::header::HeaderMap,
) {
    if url.contains('@') {
        let re = Regex::new(r".*//([^/?#\s]+)@.*").unwrap();
        if let Some(captures) = re.captures(&url) {
            if let Some(auth) = captures.get(1) {
                let b64_auth = general_purpose::STANDARD.encode(auth.as_str());
                header_map.append(
                    "Authorization",
                    ("Basic ".to_owned() + &b64_auth).parse().unwrap(),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::append_to_header::add_basic_auth_headers_conditionally;

    #[test]
    fn check_if_adding_headers_works_with_at_sign() {
        let mut header_map = reqwest::header::HeaderMap::new();
        let url = "https://user:pass@localhost:8080";

        add_basic_auth_headers_conditionally(url.to_string(), &mut header_map);

        assert_eq!(header_map.len(), 1);
        assert_eq!(
            header_map.get("Authorization").unwrap().to_str().unwrap(),
            "Basic dXNlcjpwYXNz"
        );
    }

    #[test]
    fn check_if_adding_headers_works_with_at_sign_complicated_password() {
        let mut header_map = reqwest::header::HeaderMap::new();
        let url = "https://user123123:Jm7YAT8m5YA8Forx7w6wsmUXDvcJny@localhost:8080";

        add_basic_auth_headers_conditionally(url.to_string(), &mut header_map);

        assert_eq!(header_map.len(), 1);
        assert_eq!(
            header_map.get("Authorization").unwrap().to_str().unwrap(),
            "Basic dXNlcjEyMzEyMzpKbTdZQVQ4bTVZQThGb3J4N3c2d3NtVVhEdmNKbnk="
        );
    }

    #[test]
    fn check_if_adding_headers_works_with_at_sign_and_existing_header() {
        let mut header_map = reqwest::header::HeaderMap::new();
        let url = "https://user:pass@localhost:8080";

        header_map.append("Content-Type", "application/json".parse().unwrap());

        add_basic_auth_headers_conditionally(url.to_string(), &mut header_map);

        assert_eq!(header_map.len(), 2)
    }

    #[test]
    fn check_if_nothing_happens_when_no_at_sign_present() {
        let mut header_map = reqwest::header::HeaderMap::new();
        let url = "https://user:pass@localhost:8080";

        header_map.append("Content-Type", "application/json".parse().unwrap());

        add_basic_auth_headers_conditionally(url.to_string(), &mut header_map);

        assert_eq!(header_map.len(), 2)
    }
}
