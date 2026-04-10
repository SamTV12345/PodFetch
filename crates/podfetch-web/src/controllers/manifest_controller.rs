use crate::manifest::{Manifest, build_manifest};
use crate::url_rewriting::resolve_server_url_from_headers;
use axum::Json;
use axum::http::HeaderMap;
use axum::routing::get;
use common_infrastructure::error::CustomError;
use utoipa_axum::router::OpenApiRouter;

pub async fn get_manifest(headers: HeaderMap) -> Result<Json<Manifest>, CustomError> {
    let server_url = resolve_server_url_from_headers(&headers);
    Ok(Json(build_manifest(&server_url)))
}

pub fn get_manifest_router() -> OpenApiRouter {
    OpenApiRouter::new().route("/manifest.json", get(get_manifest))
}

#[cfg(test)]
mod tests {
    use crate::test_support::tests::handle_test_startup;
    use axum::http::HeaderMap;
    use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
    use serde_json::Value;
    use serial_test::serial;

    fn assert_client_error_status(status: u16) {
        assert!(
            (400..500).contains(&status),
            "expected 4xx status, got {status}"
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_get_manifest_returns_base_structure() {
        let server = handle_test_startup().await;

        let response = server.test_server.get("/manifest.json").await;
        assert_eq!(response.status_code(), 200);

        let manifest = response.json::<Value>();
        assert_eq!(manifest["name"], "PodFetch");
        assert_eq!(manifest["short_name"], "PodFetch");
        assert_eq!(manifest["display"], "fullscreen");
        assert_eq!(manifest["orientation"], "landscape");
        assert!(manifest["icons"].as_array().is_some());
        assert!(!manifest["icons"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn test_get_manifest_uses_forwarded_headers_for_urls() {
        let mut server = handle_test_startup().await;
        server
            .test_server
            .add_header("x-forwarded-host", "manifest.example.com");
        server.test_server.add_header("x-forwarded-proto", "https");
        server.test_server.add_header("x-forwarded-prefix", "/ui");

        let response = server.test_server.get("/manifest.json").await;
        assert_eq!(response.status_code(), 200);

        let manifest = response.json::<Value>();
        assert_eq!(manifest["start_url"], "https://manifest.example.com/ui/");
        assert_eq!(
            manifest["icons"][0]["src"],
            "https://manifest.example.com/ui/ui/logo.png"
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_get_manifest_uses_first_forwarded_host_when_multiple_are_present() {
        let mut server = handle_test_startup().await;
        server
            .test_server
            .add_header("x-forwarded-host", "first.example.com, second.example.com");
        server.test_server.add_header("x-forwarded-proto", "https");

        let response = server.test_server.get("/manifest.json").await;
        assert_eq!(response.status_code(), 200);

        let manifest = response.json::<Value>();
        assert!(
            manifest["start_url"]
                .as_str()
                .unwrap()
                .starts_with("https://first.example.com/")
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_get_manifest_falls_back_to_env_server_url_when_host_headers_missing() {
        let server = handle_test_startup().await;

        let response = server.test_server.get("/manifest.json").await;
        assert_eq!(response.status_code(), 200);

        let manifest = response.json::<Value>();
        // HTTP requests from the test server include a Host header; resolver prefers request host.
        assert!(
            manifest["start_url"]
                .as_str()
                .unwrap()
                .starts_with("http://localhost")
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_get_manifest_direct_handler_without_headers_uses_env_fallback() {
        let response = super::get_manifest(HeaderMap::new()).await.unwrap();
        assert_eq!(response.0.start_url, ENVIRONMENT_SERVICE.server_url);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_manifest_returns_client_error_for_wrong_http_method() {
        let server = handle_test_startup().await;

        let response = server.test_server.post("/manifest.json").await;
        assert_client_error_status(response.status_code().as_u16());
    }
}
