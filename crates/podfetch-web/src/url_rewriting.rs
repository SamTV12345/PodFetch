//! URL rewriting utilities for adapting internal URLs to external-facing URLs.
//!
//! This module provides functions to rewrite URLs in DTOs from internal server URLs
//! to the actual server URL as seen by the client (handling reverse proxies, etc.).

use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
use http::HeaderMap;
use url::Url;

/// Rewrites a URL from the old server base to a new server base.
///
/// This handles:
/// - URLs that start with the old base (replaces prefix)
/// - Relative local paths (prepends new base)
/// - Absolute localhost URLs (rewrites if path is local)
/// - External URLs (leaves unchanged)
///
/// # Arguments
/// * `url` - The URL to rewrite
/// * `old_base` - The original server URL (e.g., "http://localhost:8080/")
/// * `new_base` - The new server URL (e.g., "https://podfetch.example.com/")
///
/// # Returns
/// The rewritten URL as a String
pub fn rewrite_url(url: &str, old_base: &str, new_base: &str) -> String {
    let old_base = normalize_server_url(old_base);
    let new_base = normalize_server_url(new_base);

    // If URL starts with old base, simple replacement
    if let Some(remainder) = url.strip_prefix(&old_base) {
        return format!("{new_base}{remainder}");
    }

    // Handle URLs that start with / and are local paths
    if url.starts_with('/') && is_local_path(url) {
        return format!("{}{}", new_base, url.trim_start_matches('/'));
    }

    // Handle relative paths like "ui/image.jpg" or "./podcasts/file.mp3"
    if let Some(local_relative_path) = normalize_relative_local_path(url) {
        return format!("{}{}", new_base, local_relative_path);
    }

    // Handle absolute URLs (e.g., http://localhost:8080/podcasts/file.mp3)
    if let Ok(parsed) = Url::parse(url)
        && is_local_host(parsed.host_str(), &old_base) && is_local_path(parsed.path())
    {
        let mut rewritten = format!("{}{}", new_base, parsed.path().trim_start_matches('/'));
        if let Some(query) = parsed.query() {
            rewritten.push('?');
            rewritten.push_str(query);
        }
        return rewritten;
    }

    // Return unchanged for external URLs
    url.to_string()
}

/// Normalizes a server URL to always end with a trailing slash.
pub fn normalize_server_url(server_url: &str) -> String {
    if server_url.ends_with('/') {
        server_url.to_string()
    } else {
        format!("{server_url}/")
    }
}

/// Checks if a path is a local application path (podcasts, ui, rss, proxy).
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

/// Normalizes a relative path to a local path, if valid.
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

/// Checks if a host is considered "local" (localhost or matching the old base).
fn is_local_host(host: Option<&str>, old_base: &str) -> bool {
    let host = match host {
        Some(host) => host,
        None => return false,
    };

    // Check common localhost variants
    if host.eq_ignore_ascii_case("localhost") || host == "127.0.0.1" || host == "::1" {
        return true;
    }

    // Check if host matches the old base URL's host
    if let Ok(parsed_old) = Url::parse(old_base)
        && let Some(old_host) = parsed_old.host_str()
    {
        return host.eq_ignore_ascii_case(old_host);
    }

    false
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

/// URL rewriter for batch rewriting URLs on DTOs.
///
/// Create once with the old and new base URLs, then use to rewrite multiple URLs.
#[derive(Debug, Clone)]
pub struct UrlRewriter {
    old_base: String,
    new_base: String,
}

impl UrlRewriter {
    /// Creates a new URL rewriter.
    ///
    /// # Arguments
    /// * `old_base` - The original server URL (e.g., "http://localhost:8080/")
    /// * `new_base` - The new server URL (e.g., "https://podfetch.example.com/")
    pub fn new(old_base: impl Into<String>, new_base: impl Into<String>) -> Self {
        Self {
            old_base: normalize_server_url(&old_base.into()),
            new_base: normalize_server_url(&new_base.into()),
        }
    }

    /// Rewrites a URL from the old base to the new base.
    pub fn rewrite(&self, url: &str) -> String {
        rewrite_url(url, &self.old_base, &self.new_base)
    }

    /// Rewrites a URL in place, taking a mutable reference to a String.
    pub fn rewrite_in_place(&self, url: &mut String) {
        *url = self.rewrite(url);
    }
}

pub fn create_url_rewriter(headers: &HeaderMap) -> UrlRewriter {
    let old_base = &ENVIRONMENT_SERVICE.server_url;
    let new_base = resolve_server_url_from_headers(headers);
    UrlRewriter::new(old_base, new_base)
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::HeaderMap;
    use http::HeaderValue;

    #[test]
    fn test_rewrite_url_with_old_base_prefix() {
        let result = rewrite_url(
            "http://localhost:8080/podcasts/file.mp3",
            "http://localhost:8080/",
            "https://podfetch.example.com/",
        );
        assert_eq!(result, "https://podfetch.example.com/podcasts/file.mp3");
    }

    #[test]
    fn test_rewrite_url_relative_path() {
        let result = rewrite_url(
            "ui/default.jpg",
            "http://localhost:8080/",
            "http://new.host/",
        );
        assert_eq!(result, "http://new.host/ui/default.jpg");
    }

    #[test]
    fn test_rewrite_url_absolute_path() {
        let result = rewrite_url(
            "/podcasts/episode.mp3",
            "http://localhost:8080/",
            "https://example.com/",
        );
        assert_eq!(result, "https://example.com/podcasts/episode.mp3");
    }

    #[test]
    fn test_rewrite_url_external_unchanged() {
        let result = rewrite_url(
            "https://external.com/image.jpg",
            "http://localhost:8080/",
            "https://example.com/",
        );
        assert_eq!(result, "https://external.com/image.jpg");
    }

    #[test]
    fn test_rewrite_url_with_query_params() {
        let result = rewrite_url(
            "http://localhost:8080/rss?apiKey=123",
            "http://localhost:8080/",
            "https://example.com/",
        );
        assert_eq!(result, "https://example.com/rss?apiKey=123");
    }

    #[test]
    fn test_url_rewriter_struct() {
        let rewriter = UrlRewriter::new("http://localhost:8080", "https://example.com");
        assert_eq!(
            rewriter.rewrite("/ui/image.jpg"),
            "https://example.com/ui/image.jpg"
        );
    }

    #[test]
    fn test_create_url_rewriter_from_headers() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-host",
            HeaderValue::from_static("podfetch.example.com"),
        );
        headers.insert("x-forwarded-proto", HeaderValue::from_static("https"));
        let rewriter = create_url_rewriter(&headers);
        assert_eq!(
            rewriter.rewrite("/ui/default.jpg"),
            "https://podfetch.example.com/ui/default.jpg"
        );
    }

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
}
