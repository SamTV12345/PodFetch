//! URL rewriting utilities for adapting internal URLs to external-facing URLs.
//!
//! This module provides functions to rewrite URLs in DTOs from internal server URLs
//! to the actual server URL as seen by the client (handling reverse proxies, etc.).

use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
use http::HeaderMap;

/// Normalizes a server URL to always end with a trailing slash.
pub fn normalize_server_url(server_url: &str) -> String {
    if server_url.ends_with('/') {
        server_url.to_string()
    } else {
        format!("{server_url}/")
    }
}

fn get_header_value(headers: &HeaderMap, key: &str) -> Option<String> {
    headers
        .get(key)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(str::trim)
        .map(ToString::to_string)
        .filter(|v| !v.is_empty())
}

pub fn resolve_server_url_from_headers(headers: &HeaderMap) -> String {
    let host = get_header_value(headers, "x-forwarded-host")
        .or_else(|| get_header_value(headers, "host"))
        .or_else(|| get_header_value(headers, ":authority"));

    if host.is_none() {
        // HTTP/1.1 requires Host. If somehow none of x-forwarded-host / host /
        // :authority are present, return empty — callers building URLs will
        // emit relative paths, which is the safest degraded behaviour.
        return String::new();
    }

    let proto = get_header_value(headers, "x-forwarded-proto")
        .or_else(|| get_header_value(headers, "x-forwarded-scheme"))
        .unwrap_or_else(|| "http".to_string());

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

/// Resolve a stored image URL into a client-facing URL.
///
/// - Empty string → default image at `<server_url>/ui/default.jpg`.
/// - Absolute http/https (case-insensitive scheme) → passthrough (external remote URL).
/// - Relative path → `<server_url>/<path>` (leading slash stripped before joining).
///
/// If `server_url` is empty (e.g., no Host header present), the returned URL
/// will be root-relative (e.g., `"/ui/default.jpg"`).
pub fn resolve_image_url(stored: &str, server_url: &str) -> String {
    use common_infrastructure::runtime::DEFAULT_IMAGE_URL;
    let base = normalize_server_url(server_url);
    if stored.is_empty() {
        return format!("{base}{DEFAULT_IMAGE_URL}");
    }
    let lower = stored.to_ascii_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") {
        return stored.to_string();
    }
    format!("{base}{}", stored.trim_start_matches('/'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::HeaderMap;
    use http::HeaderValue;

    #[test]
    fn test_normalize_server_url() {
        assert_eq!(
            normalize_server_url("https://example.com"),
            "https://example.com/"
        );
        assert_eq!(
            normalize_server_url("https://example.com/"),
            "https://example.com/"
        );
    }

    #[test]
    fn resolve_image_url_empty_stored_returns_default() {
        let result = resolve_image_url("", "https://example.com/");
        assert_eq!(result, "https://example.com/ui/default.jpg");
    }

    #[test]
    fn resolve_image_url_absolute_http_passes_through() {
        let result = resolve_image_url("http://remote.example/cover.jpg", "https://example.com/");
        assert_eq!(result, "http://remote.example/cover.jpg");
    }

    #[test]
    fn resolve_image_url_absolute_https_passes_through() {
        let result = resolve_image_url("https://remote.example/cover.jpg", "https://example.com/");
        assert_eq!(result, "https://remote.example/cover.jpg");
    }

    #[test]
    fn resolve_image_url_relative_path_gets_prefixed() {
        let result = resolve_image_url("podcasts/foo/image.jpg", "https://example.com/");
        assert_eq!(result, "https://example.com/podcasts/foo/image.jpg");
    }

    #[test]
    fn resolve_image_url_leading_slash_stripped_before_prefix() {
        let result = resolve_image_url("/podcasts/foo/image.jpg", "https://example.com/");
        assert_eq!(result, "https://example.com/podcasts/foo/image.jpg");
    }

    #[test]
    fn resolve_image_url_uppercase_scheme_passes_through() {
        let result = resolve_image_url("HTTP://remote.example/cover.jpg", "https://example.com/");
        assert_eq!(result, "HTTP://remote.example/cover.jpg");
    }

    #[test]
    fn resolve_image_url_server_url_without_trailing_slash_is_normalized() {
        let result = resolve_image_url("podcasts/foo/image.jpg", "https://example.com");
        assert_eq!(result, "https://example.com/podcasts/foo/image.jpg");
    }

    #[test]
    fn resolve_image_url_empty_server_url_returns_root_relative() {
        let result = resolve_image_url("", "");
        assert_eq!(result, "/ui/default.jpg");
    }

    #[test]
    fn resolve_server_url_from_headers_returns_empty_when_no_host_headers() {
        let headers = HeaderMap::new();
        let result = resolve_server_url_from_headers(&headers);
        assert_eq!(result, "");
    }

    #[test]
    fn resolve_server_url_from_headers_uses_x_forwarded_host() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-host",
            HeaderValue::from_static("podfetch.example.com"),
        );
        headers.insert("x-forwarded-proto", HeaderValue::from_static("https"));
        let result = resolve_server_url_from_headers(&headers);
        assert_eq!(result, "https://podfetch.example.com/");
    }
}
