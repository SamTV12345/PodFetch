use crate::app_state::AppState;
use crate::auth::{AuthControllerError, parse_basic_auth};
use crate::services::user_auth::service::UserAuthService;
use axum::extract::{Request, State};
use axum::http::header::AUTHORIZATION;
use axum::middleware::Next;
use axum::response::Response;
use common_infrastructure::config::{EnvironmentService, ReverseProxyConfig};
use common_infrastructure::error::ErrorSeverity::Warning;
use common_infrastructure::error::{CustomError, CustomErrorInner, ErrorSeverity};
use common_infrastructure::http::get_async_sync_client;
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
use jsonwebtoken::jwk::{Jwk, JwkSet};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use podfetch_domain::user::User;
use serde_json::Value;
use sha256::digest;
use std::fmt::Display;
use std::sync::{Arc, PoisonError, RwLock};
use tracing::info;

enum AuthType {
    Basic,
    Oidc,
    Proxy,
    None,
}

pub struct AuthFilter;

pub async fn handle_basic_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, CustomError> {
    let user = handle_auth_internal(&mut request, AuthType::Basic, &state).await?;
    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}

/// The identity provider's signing keys, fetched lazily and refreshed when a
/// token is signed with a key we do not know yet (key rotation).
static JWKS: RwLock<Option<Arc<JwkSet>>> = RwLock::new(None);

fn unavailable(reason: impl Display) -> CustomError {
    tracing::error!("OIDC provider unavailable: {reason}");
    CustomErrorInner::Unknown(ErrorSeverity::Error).into()
}

fn forbidden() -> CustomError {
    CustomErrorInner::Forbidden(Warning).into()
}

pub async fn get_jwks(jwks_uri: &str) -> Result<JwkSet, CustomError> {
    get_async_sync_client(&ENVIRONMENT_SERVICE)
        .build()
        .map_err(unavailable)?
        .get(jwks_uri)
        .send()
        .await
        .map_err(unavailable)?
        .error_for_status()
        .map_err(unavailable)?
        .json::<JwkSet>()
        .await
        .map_err(unavailable)
}

/// Returns the key set, fetching it if it is missing or `force_refresh` is set.
async fn jwks(jwks_uri: &str, force_refresh: bool) -> Result<Arc<JwkSet>, CustomError> {
    if !force_refresh
        && let Some(cached) = JWKS.read().unwrap_or_else(PoisonError::into_inner).clone()
    {
        return Ok(cached);
    }
    let fresh = Arc::new(get_jwks(jwks_uri).await?);
    *JWKS.write().unwrap_or_else(PoisonError::into_inner) = Some(fresh.clone());
    Ok(fresh)
}

/// Picks the key a token was signed with: the one matching its `kid`, or the
/// first key when the token does not name one.
fn select_jwk<'a>(jwks: &'a JwkSet, kid: Option<&str>) -> Option<&'a Jwk> {
    match kid {
        Some(kid) => jwks.find(kid),
        None => jwks.keys.first(),
    }
}

/// Verifies `token` against `jwk` and returns its `preferred_username` claim.
fn verify_token(token: &str, jwk: &Jwk, client_id: &str) -> Result<String, CustomError> {
    let key = DecodingKey::from_jwk(jwk).map_err(|e| {
        tracing::error!("Unusable JWK: {e}");
        forbidden()
    })?;
    let Some(alg) = jwk
        .common
        .key_algorithm
        .and_then(|alg| Algorithm::try_from(alg).ok())
    else {
        tracing::error!("JWK has no supported signing algorithm");
        return Err(forbidden());
    };
    let mut validation = Validation::new(alg);
    validation.set_audience(&[client_id]);

    let decoded = decode::<Value>(token, &key, &validation).map_err(|e| {
        tracing::debug!("Rejected OIDC token: {e}");
        forbidden()
    })?;
    decoded
        .claims
        .get("preferred_username")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(forbidden)
}

/// Extracts the token of an `Authorization: Bearer <token>` header.
fn bearer_token(req: &Request) -> Result<&str, CustomError> {
    let header = req
        .headers()
        .get(AUTHORIZATION)
        .ok_or_else(|| CustomErrorInner::UnAuthorized("Unauthorized".to_owned(), Warning))?
        .to_str()
        .map_err(|_| forbidden())?;
    Ok(header.strip_prefix("Bearer ").unwrap_or(header))
}

pub async fn handle_oidc_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, CustomError> {
    let user = handle_auth_internal(&mut request, AuthType::Oidc, &state).await?;
    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}

pub async fn handle_proxy_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, CustomError> {
    let user = handle_auth_internal(&mut request, AuthType::Proxy, &state).await?;
    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}

pub async fn handle_no_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, CustomError> {
    let user = handle_auth_internal(&mut request, AuthType::None, &state).await?;
    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}

async fn handle_auth_internal(
    req: &mut Request,
    auth_type: AuthType,
    state: &AppState,
) -> Result<User, CustomError> {
    match auth_type {
        AuthType::Basic => {
            AuthFilter::handle_basic_auth_internal(req, state.user_auth_service.as_ref())
        }
        AuthType::Oidc => {
            AuthFilter::handle_oidc_auth_internal(
                req,
                state.user_auth_service.as_ref(),
                state.environment.as_ref(),
            )
            .await
        }
        AuthType::Proxy => {
            let config = state
                .environment
                .reverse_proxy_config
                .as_ref()
                .ok_or_else(|| unavailable("Reverse proxy auth is enabled but not configured"))?;
            AuthFilter::handle_proxy_auth_internal(req, config, state.user_auth_service.as_ref())
        }
        AuthType::None => state.user_auth_service.ensure_admin_user(),
    }
}

impl AuthFilter {
    fn map_auth_error(error: AuthControllerError<CustomError>) -> CustomError {
        match error {
            AuthControllerError::Forbidden => CustomErrorInner::Forbidden(Warning).into(),
            AuthControllerError::Unauthorized(message) => {
                CustomErrorInner::UnAuthorized(message, Warning).into()
            }
            AuthControllerError::Service(error) => error,
        }
    }

    pub fn extract_basic_auth(auth: &str) -> Result<(String, String), CustomError> {
        parse_basic_auth::<CustomError>(auth).map_err(Self::map_auth_error)
    }

    pub fn basic_auth_login(rq: &str) -> Result<(String, String), CustomError> {
        let (u, p) = Self::extract_basic_auth(rq)?;

        Ok((u.clone(), p.clone()))
    }

    fn handle_basic_auth_internal(
        req: &Request,
        user_auth_service: &UserAuthService,
    ) -> Result<User, CustomError> {
        let opt_auth_header = req.headers().get("Authorization");
        match opt_auth_header {
            Some(header) => match header.to_str() {
                Ok(auth) => {
                    let (user, password) = AuthFilter::extract_basic_auth(auth)?;

                    let found_user = user_auth_service
                        .find_by_username(&user)
                        .map_err(|_| CustomErrorInner::Forbidden(Warning))?;

                    if let Some(password_from_user) = &found_user.password {
                        if password_from_user == &digest(password) {
                            Ok(found_user)
                        } else {
                            Err(CustomErrorInner::Forbidden(Warning).into())
                        }
                    } else {
                        Err(CustomErrorInner::Forbidden(Warning).into())
                    }
                }
                Err(_) => Err(CustomErrorInner::Forbidden(Warning).into()),
            },
            None => Err(CustomErrorInner::Forbidden(Warning).into()),
        }
    }

    async fn handle_oidc_auth_internal(
        req: &mut Request,
        user_auth_service: &UserAuthService,
        environment: &EnvironmentService,
    ) -> Result<User, CustomError> {
        let oidc = environment
            .oidc_config
            .as_ref()
            .ok_or_else(|| unavailable("OIDC is enabled but not configured"))?;
        let token = bearer_token(req)?;
        let kid = decode_header(token).map_err(|_| forbidden())?.kid;

        let mut keys = jwks(&oidc.jwks_uri, false).await?;
        if select_jwk(&keys, kid.as_deref()).is_none() {
            // The provider may have rotated its keys since we fetched them.
            keys = jwks(&oidc.jwks_uri, true).await?;
        }
        let Some(jwk) = select_jwk(&keys, kid.as_deref()) else {
            tracing::error!("No matching JWK found at {}", oidc.jwks_uri);
            return Err(forbidden());
        };
        let username = verify_token(token, jwk, &oidc.client_id)?;

        let user = match user_auth_service.find_by_username(&username) {
            Ok(user) => user,
            // The user is authenticated by the provider, so onboard them.
            Err(_) => user_auth_service.create_user(username, "user".to_owned(), None, false)?,
        };
        // The UI needs an api key to open its socket.io connection.
        user_auth_service.ensure_api_key(user)
    }

    fn handle_proxy_auth_internal(
        req: &Request,
        reverse_proxy_config: &ReverseProxyConfig,
        user_auth_service: &UserAuthService,
    ) -> Result<User, CustomError> {
        let Some(header) = req.headers().get(&reverse_proxy_config.header_name) else {
            info!("Reverse proxy is enabled but no header is provided");
            return Err(forbidden());
        };
        let username = header.to_str().map_err(|_| forbidden())?;

        match user_auth_service.find_by_username(username) {
            Ok(user) => Ok(user),
            Err(_) if reverse_proxy_config.auto_sign_up => {
                user_auth_service.create_user(username.to_owned(), "user".to_owned(), None, false)
            }
            Err(_) => Err(forbidden()),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::app_state::AppState;
    use axum::extract::Request;

    use crate::auth_middleware::AuthFilter;

    use crate::test_support::tests::handle_test_startup;
    use crate::test_utils::test::create_random_user;
    use common_infrastructure::config::ReverseProxyConfig;
    use serial_test::serial;

    fn app_state() -> AppState {
        AppState::new()
    }

    #[test]
    #[serial]
    fn test_basic_auth_login() {
        let result = AuthFilter::extract_basic_auth("Basic dGVzdDp0ZXN0");
        assert!(result.is_ok());
        let (u, p) = result.unwrap();
        assert_eq!(u, "test");
        assert_eq!(p, "test");
    }

    #[test]
    #[serial]
    fn test_basic_auth_login_with_special_characters() {
        let result = AuthFilter::extract_basic_auth("Basic dGVzdCTDvMOWOnRlc3Q=");
        assert!(result.is_ok());
        let (u, p) = result.unwrap();
        assert_eq!(u, "test$üÖ");
        assert_eq!(p, "test");
    }

    #[serial]
    #[tokio::test]
    async fn test_proxy_auth_with_no_header_in_request_auto_sign_up() {
        let _router = handle_test_startup().await;
        let rv_config = ReverseProxyConfig {
            header_name: "X-Auth".to_string(),
            auto_sign_up: true,
        };

        let req = Request::builder()
            .header("Content-Type", "text/plain")
            .body("".into())
            .unwrap();
        let state = app_state();
        let result = AuthFilter::handle_proxy_auth_internal(
            &req,
            &rv_config,
            state.user_auth_service.as_ref(),
        );

        assert!(result.is_err());
    }

    #[serial]
    #[tokio::test]
    async fn test_proxy_auth_with_header_in_request_auto_sign_up() {
        let _server = handle_test_startup().await;
        let rv_config = ReverseProxyConfig {
            header_name: "X-Auth".to_string(),
            auto_sign_up: false,
        };
        let req = Request::builder()
            .header("Content-Type", "text/plain")
            .header("X-Auth", "test1")
            .body("".into())
            .unwrap();
        let state = app_state();
        let result = AuthFilter::handle_proxy_auth_internal(
            &req,
            &rv_config,
            state.user_auth_service.as_ref(),
        );

        assert!(result.is_err());
    }

    #[serial]
    #[tokio::test]
    async fn test_proxy_auth_with_header_in_request_auto_sign_up_false() {
        let _server = handle_test_startup().await;
        let rv_config = ReverseProxyConfig {
            header_name: "X-Auth".to_string(),
            auto_sign_up: true,
        };

        let req = Request::builder()
            .header("Content-Type", "text/plain")
            .header("X-Auth", "test")
            .body("".into())
            .unwrap();
        let state = app_state();
        let result = AuthFilter::handle_proxy_auth_internal(
            &req,
            &rv_config,
            state.user_auth_service.as_ref(),
        );

        assert!(result.is_ok());
    }

    #[serial]
    #[tokio::test]
    async fn test_basic_auth_no_header() {
        let _router = handle_test_startup().await;

        let req = Request::builder()
            .header("Content-Type", "text/plain")
            .body("".into())
            .unwrap();
        let state = app_state();
        let result = AuthFilter::handle_basic_auth_internal(&req, state.user_auth_service.as_ref());

        assert!(result.is_err());
    }

    #[serial]
    #[tokio::test]
    async fn test_basic_auth_header_no_user() {
        let _router = handle_test_startup().await;

        let req = Request::builder()
            .header("Content-Type", "text/plain")
            .header("Authorization", "Bearer dGVzdDp0ZXN0")
            .body("".into())
            .unwrap();
        let state = app_state();
        let result = AuthFilter::handle_basic_auth_internal(&req, state.user_auth_service.as_ref());

        assert!(result.is_err());
    }

    #[serial]
    #[tokio::test]
    async fn test_basic_auth_header_other_user() {
        // given
        let _router = handle_test_startup().await;
        let state = app_state();
        state
            .user_admin_service
            .create_user(create_random_user())
            .unwrap();

        let req = Request::builder()
            .header("Content-Type", "text/plain")
            .header("Authorization", "Bearer dGVzdDp0ZXN0")
            .body("".into())
            .unwrap();

        // when
        let result = AuthFilter::handle_basic_auth_internal(&req, state.user_auth_service.as_ref());

        // then
        assert!(result.is_err());
    }

    #[serial]
    #[tokio::test]
    async fn test_basic_auth_header_correct_user_wrong_password() {
        // given
        let _router = handle_test_startup().await;
        let state = app_state();
        state
            .user_admin_service
            .create_user(create_random_user())
            .unwrap();

        let req = Request::builder()
            .header("Content-Type", "text/plain")
            .header("Authorization", "Bearer dGVzdHVzZXI6dGVzdA==")
            .body("".into())
            .unwrap();
        // when
        let result = AuthFilter::handle_basic_auth_internal(&req, state.user_auth_service.as_ref());

        // then
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod oidc_tests {
    use super::*;
    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use jsonwebtoken::{EncodingKey, Header, encode};
    use serde_json::json;

    const CLIENT_ID: &str = "podfetch";

    fn key_set() -> JwkSet {
        let jwk = |kid: &str, secret: &[u8]| json!({"kty": "oct", "kid": kid, "alg": "HS256", "k": URL_SAFE_NO_PAD.encode(secret)});
        serde_json::from_value(
            json!({"keys": [jwk("old", b"old-secret"), jwk("new", b"new-secret")]}),
        )
        .unwrap()
    }

    fn token(kid: Option<&str>, secret: &[u8], aud: &str) -> String {
        let mut header = Header::new(Algorithm::HS256);
        header.kid = kid.map(str::to_owned);
        let claims = json!({
            "preferred_username": "alice",
            "aud": aud,
            "exp": chrono::Utc::now().timestamp() + 600,
        });
        encode(&header, &claims, &EncodingKey::from_secret(secret)).unwrap()
    }

    #[test]
    fn selects_the_key_named_by_kid() {
        let keys = key_set();
        assert_eq!(
            select_jwk(&keys, Some("new"))
                .unwrap()
                .common
                .key_id
                .as_deref(),
            Some("new")
        );
        assert!(select_jwk(&keys, Some("unknown")).is_none());
        assert_eq!(
            select_jwk(&keys, None).unwrap().common.key_id.as_deref(),
            Some("old")
        );
    }

    #[test]
    fn accepts_a_token_signed_with_a_rotated_key() {
        let keys = key_set();
        let token = token(Some("new"), b"new-secret", CLIENT_ID);
        let jwk = select_jwk(&keys, Some("new")).unwrap();
        assert_eq!(verify_token(&token, jwk, CLIENT_ID).unwrap(), "alice");
    }

    #[test]
    fn rejects_wrong_signature_and_audience() {
        let keys = key_set();
        let jwk = select_jwk(&keys, Some("new")).unwrap();
        assert!(verify_token(&token(Some("new"), b"forged", CLIENT_ID), jwk, CLIENT_ID).is_err());
        assert!(verify_token(&token(Some("new"), b"new-secret", "other"), jwk, CLIENT_ID).is_err());
    }

    #[test]
    fn bearer_prefix_is_optional() {
        let req = |value: &str| {
            Request::builder()
                .header(AUTHORIZATION, value)
                .body(axum::body::Body::empty())
                .unwrap()
        };
        assert_eq!(bearer_token(&req("Bearer abc")).unwrap(), "abc");
        assert_eq!(bearer_token(&req("abc")).unwrap(), "abc");
        let missing = Request::builder().body(axum::body::Body::empty()).unwrap();
        assert!(bearer_token(&missing).is_err());
    }
}
