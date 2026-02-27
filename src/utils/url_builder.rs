use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use axum::http::HeaderMap;
use url::Url;

fn get_header_value(headers: &HeaderMap, key: &str) -> Option<String> {
    headers
        .get(key)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(str::trim)
        .map(ToString::to_string)
        .filter(|v| !v.is_empty())
}

pub fn normalize_server_url(server_url: &str) -> String {
    if server_url.ends_with('/') {
        server_url.to_string()
    } else {
        format!("{server_url}/")
    }
}

pub fn resolve_server_url_from_headers(headers: &HeaderMap) -> String {
    let host = get_header_value(headers, "x-forwarded-host")
        .or_else(|| get_header_value(headers, "host"))
        .or_else(|| get_header_value(headers, ":authority"));

    if host.is_none() {
        return ENVIRONMENT_SERVICE.server_url.clone();
    }

    let proto = get_header_value(headers, "x-forwarded-proto")
        .or_else(|| get_header_value(headers, "x-forwarded-scheme"))
        .unwrap_or_else(|| {
            if ENVIRONMENT_SERVICE.server_url.starts_with("https://") {
                "https".to_string()
            } else {
                "http".to_string()
            }
        });

    let prefix = get_header_value(headers, "x-forwarded-prefix")
        .or_else(|| ENVIRONMENT_SERVICE.sub_directory.clone())
        .unwrap_or_default();
    let cleaned_prefix = prefix.trim_matches('/');

    let mut base = format!("{}://{}", proto, host.unwrap());
    if !cleaned_prefix.is_empty() {
        base.push('/');
        base.push_str(cleaned_prefix);
    }

    normalize_server_url(&base)
}

pub fn rewrite_env_server_url_prefix(url: &str, resolved_server_url: &str) -> String {
    let old_base = normalize_server_url(&ENVIRONMENT_SERVICE.server_url);
    let new_base = normalize_server_url(resolved_server_url);
    if let Some(remainder) = url.strip_prefix(&old_base) {
        format!("{new_base}{remainder}")
    } else if url.starts_with('/') && is_local_path(url) {
        format!("{}{}", new_base, url.trim_start_matches('/'))
    } else if let Some(local_relative_path) = normalize_relative_local_path(url) {
        format!("{}{}", new_base, local_relative_path)
    } else if let Ok(parsed) = Url::parse(url) {
        if is_local_host(parsed.host_str()) && is_local_path(parsed.path()) {
            let mut rewritten = format!("{}{}", new_base, parsed.path().trim_start_matches('/'));
            if let Some(query) = parsed.query() {
                rewritten.push('?');
                rewritten.push_str(query);
            }
            rewritten
        } else {
            url.to_string()
        }
    } else {
        url.to_string()
    }
}

fn normalize_relative_local_path(path: &str) -> Option<String> {
    let normalized = path.trim().trim_start_matches("./").trim_start_matches('/');
    if normalized.is_empty() {
        return None;
    }

    let normalized_with_slash = format!("/{normalized}");
    if is_local_path(&normalized_with_slash) {
        Some(normalized.to_string())
    } else {
        None
    }
}

fn is_local_path(path: &str) -> bool {
    path == "/rss"
        || path.starts_with("/rss/")
        || path == "/proxy"
        || path.starts_with("/proxy/")
        || path == "/podcasts"
        || path.starts_with("/podcasts/")
        || path == "/ui"
        || path.starts_with("/ui/")
}

fn is_local_host(host: Option<&str>) -> bool {
    let host = match host {
        Some(host) => host,
        None => return false,
    };

    if host.eq_ignore_ascii_case("localhost") || host == "127.0.0.1" || host == "::1" {
        return true;
    }

    if let Ok(parsed_env_url) = Url::parse(&ENVIRONMENT_SERVICE.server_url)
        && let Some(env_host) = parsed_env_url.host_str()
    {
        return host.eq_ignore_ascii_case(env_host);
    }

    false
}

pub fn build_ws_url_from_server_url(server_url: &str) -> String {
    let normalized = normalize_server_url(server_url);
    if let Ok(mut parsed) = Url::parse(&normalized) {
        let ws_scheme = if parsed.scheme() == "https" {
            "wss"
        } else {
            "ws"
        };
        if parsed.set_scheme(ws_scheme).is_ok() {
            let mut path = parsed.path().trim_end_matches('/').to_string();
            path.push('/');
            path.push_str("socket.io");
            parsed.set_path(&path);
            return parsed.to_string();
        }
    }

    ENVIRONMENT_SERVICE.ws_url.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn test_normalize_server_url_adds_trailing_slash() {
        assert_eq!(
            normalize_server_url("https://example.com/ui"),
            "https://example.com/ui/"
        );
        assert_eq!(
            normalize_server_url("https://example.com/ui/"),
            "https://example.com/ui/"
        );
    }

    #[test]
    fn test_resolve_server_url_from_headers_uses_forwarded_values() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-host",
            HeaderValue::from_static("podfetch.example.com"),
        );
        headers.insert("x-forwarded-proto", HeaderValue::from_static("https"));
        headers.insert("x-forwarded-prefix", HeaderValue::from_static("/ui"));

        let resolved = resolve_server_url_from_headers(&headers);
        assert_eq!(resolved, "https://podfetch.example.com/ui/");
    }

    #[test]
    fn test_resolve_server_url_from_headers_falls_back_to_env_when_host_missing() {
        let headers = HeaderMap::new();
        assert_eq!(
            resolve_server_url_from_headers(&headers),
            ENVIRONMENT_SERVICE.server_url
        );
    }

    #[test]
    fn test_rewrite_env_server_url_prefix_for_env_based_url() {
        let old_url = format!("{}podcasts/a/b/image.jpg", ENVIRONMENT_SERVICE.server_url);
        let rewritten = rewrite_env_server_url_prefix(&old_url, "https://mobile.example.com/ui/");
        assert_eq!(
            rewritten,
            "https://mobile.example.com/ui/podcasts/a/b/image.jpg"
        );
    }

    #[test]
    fn test_rewrite_env_server_url_prefix_for_absolute_localhost_url() {
        let old_url = "http://localhost:8080/podcasts/MeTacheles/image.jpg";
        let rewritten = rewrite_env_server_url_prefix(old_url, "http://localhost:5173/");
        assert_eq!(
            rewritten,
            "http://localhost:5173/podcasts/MeTacheles/image.jpg"
        );
    }

    #[test]
    fn test_rewrite_env_server_url_prefix_keeps_query_parameters() {
        let old_url = "http://127.0.0.1:8080/proxy/podcast?episodeId=abc&apiKey=123";
        let rewritten = rewrite_env_server_url_prefix(old_url, "https://podfetch.example.com/ui/");
        assert_eq!(
            rewritten,
            "https://podfetch.example.com/ui/proxy/podcast?episodeId=abc&apiKey=123"
        );
    }

    #[test]
    fn test_rewrite_env_server_url_prefix_does_not_touch_remote_urls() {
        let remote = "https://cdn.example.com/image.jpg";
        assert_eq!(
            rewrite_env_server_url_prefix(remote, "http://localhost:5173/"),
            remote
        );
    }

    #[test]
    fn test_rewrite_env_server_url_prefix_for_relative_ui_path() {
        let rewritten = rewrite_env_server_url_prefix("ui/default.jpg", "http://localhost:5173/");
        assert_eq!(rewritten, "http://localhost:5173/ui/default.jpg");
    }

    #[test]
    fn test_rewrite_env_server_url_prefix_for_relative_podcast_path() {
        let rewritten = rewrite_env_server_url_prefix(
            "./podcasts/test-podcast/image.jpg",
            "https://mobile.example.com/ui/",
        );
        assert_eq!(
            rewritten,
            "https://mobile.example.com/ui/podcasts/test-podcast/image.jpg"
        );
    }

    #[test]
    fn test_build_ws_url_from_server_url_with_sub_path() {
        let ws = build_ws_url_from_server_url("https://podfetch.example.com/ui/");
        assert_eq!(ws, "wss://podfetch.example.com/ui/socket.io");
    }

    #[test]
    fn test_build_ws_url_from_server_url_from_http() {
        let ws = build_ws_url_from_server_url("http://localhost:5173/");
        assert_eq!(ws, "ws://localhost:5173/socket.io");
    }
}
