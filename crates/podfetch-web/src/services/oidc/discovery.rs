//! Resolve the OIDC authorization and token endpoints from the provider's
//! discovery document (`<OIDC_AUTHORITY>/.well-known/openid-configuration`).
//!
//! Providers do not agree on endpoint paths (Keycloak:
//! `.../openid-connect/auth` + `.../openid-connect/token`, Pocket ID:
//! `/authorize` + `/api/oidc/token`, Authelia: `/api/oidc/authorization` +
//! `/api/oidc/token`), so guessing them from `OIDC_AUTHORITY` only works for
//! some of them. Explicit `OIDC_AUTHORIZATION_ENDPOINT` / `OIDC_TOKEN_ENDPOINT`
//! always win; without discovery the previous behaviour stays in place.

use common_infrastructure::config::{OidcConfig, OidcEndpoints, set_discovered_oidc_endpoints};
use serde::Deserialize;
use std::time::Duration;

#[derive(Deserialize)]
struct DiscoveryDocument {
    authorization_endpoint: Option<String>,
    token_endpoint: Option<String>,
}

const ATTEMPTS: u32 = 5;
const RETRY_DELAY: Duration = Duration::from_secs(10);

/// Parse a discovery document; both endpoints must be present.
pub fn parse_discovery_document(body: &str) -> Option<OidcEndpoints> {
    let document: DiscoveryDocument = serde_json::from_str(body).ok()?;
    let non_empty = |value: Option<String>| value.filter(|v| !v.trim().is_empty());
    Some(OidcEndpoints {
        authorization_endpoint: non_empty(document.authorization_endpoint)?,
        token_endpoint: non_empty(document.token_endpoint)?,
    })
}

pub async fn fetch_discovery_document(
    client: &reqwest::Client,
    url: &str,
) -> Result<OidcEndpoints, String> {
    let response = client
        .get(url)
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await
        .map_err(|err| err.to_string())?;
    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }
    let body = response.text().await.map_err(|err| err.to_string())?;
    parse_discovery_document(&body)
        .ok_or_else(|| "document has no authorization_endpoint/token_endpoint".to_string())
}

/// Discover the endpoints in the background (with retries, so a provider
/// that starts after PodFetch is still picked up). Until then - or if the
/// provider has no discovery document - the configured/legacy endpoints are
/// served to the clients.
pub fn spawn_endpoint_discovery(config: &OidcConfig) {
    if OidcConfig::endpoints_configured_explicitly() {
        tracing::info!(
            "OIDC: using configured endpoints (authorization: {}, token: {})",
            config.authorization_endpoint,
            config.token_endpoint
        );
        return;
    }
    let url = config.discovery_url();
    let legacy = (
        config.authorization_endpoint.clone(),
        config.token_endpoint.clone(),
    );
    tokio::spawn(async move {
        let client = match reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
        {
            Ok(client) => client,
            Err(err) => {
                tracing::warn!("OIDC: could not create HTTP client for discovery: {err}");
                return;
            }
        };
        for attempt in 1..=ATTEMPTS {
            match fetch_discovery_document(&client, &url).await {
                Ok(endpoints) => {
                    tracing::info!(
                        "OIDC: discovered endpoints via {url} (authorization: {}, token: {})",
                        endpoints.authorization_endpoint,
                        endpoints.token_endpoint
                    );
                    set_discovered_oidc_endpoints(endpoints);
                    return;
                }
                Err(err) if attempt < ATTEMPTS => {
                    tracing::debug!("OIDC: discovery via {url} failed ({err}), retrying");
                    tokio::time::sleep(RETRY_DELAY).await;
                }
                Err(err) => {
                    tracing::warn!(
                        "OIDC: no discovery document at {url} ({err}). Using authorization endpoint {} \
                         and token endpoint {}. Set OIDC_AUTHORITY to your provider's issuer URL or \
                         configure OIDC_AUTHORIZATION_ENDPOINT and OIDC_TOKEN_ENDPOINT.",
                        legacy.0,
                        legacy.1
                    );
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::parse_discovery_document;
    use common_infrastructure::config::OidcConfig;

    #[test]
    fn parses_pocket_id_style_document() {
        let endpoints = parse_discovery_document(
            r#"{"issuer":"https://id.example.com","authorization_endpoint":"https://id.example.com/authorize","token_endpoint":"https://id.example.com/api/oidc/token","jwks_uri":"https://id.example.com/.well-known/jwks.json"}"#,
        )
        .unwrap();
        assert_eq!(
            endpoints.authorization_endpoint,
            "https://id.example.com/authorize"
        );
        assert_eq!(
            endpoints.token_endpoint,
            "https://id.example.com/api/oidc/token"
        );
    }

    #[test]
    fn rejects_incomplete_documents() {
        assert!(
            parse_discovery_document(r#"{"authorization_endpoint":"https://x/auth"}"#).is_none()
        );
        assert!(
            parse_discovery_document(
                r#"{"authorization_endpoint":"","token_endpoint":"https://x/t"}"#
            )
            .is_none()
        );
        assert!(parse_discovery_document("not json").is_none());
    }

    #[test]
    fn legacy_token_endpoint_is_sibling_of_authorization_endpoint() {
        let keycloak = OidcConfig::legacy_endpoints(
            "https://kc.example.com/realms/master/protocol/openid-connect/auth",
        );
        assert_eq!(
            keycloak.authorization_endpoint,
            "https://kc.example.com/realms/master/protocol/openid-connect/auth"
        );
        assert_eq!(
            keycloak.token_endpoint,
            "https://kc.example.com/realms/master/protocol/openid-connect/token"
        );
        // Trailing slash doesn't change the result.
        assert_eq!(
            OidcConfig::legacy_endpoints(
                "https://kc.example.com/realms/master/protocol/openid-connect/auth/"
            )
            .token_endpoint,
            keycloak.token_endpoint
        );
    }

    #[test]
    fn discovery_url_is_below_the_issuer() {
        let oidc = OidcConfig {
            authority: "https://id.example.com/".to_string(),
            client_id: "podfetch".to_string(),
            redirect_uri: "https://podfetch.example.com/ui/".to_string(),
            scope: "openid".to_string(),
            jwks_uri: String::new(),
            refresh_interval: 1,
            authorization_endpoint: String::new(),
            token_endpoint: String::new(),
        };
        assert_eq!(
            oidc.discovery_url(),
            "https://id.example.com/.well-known/openid-configuration"
        );
    }
}
