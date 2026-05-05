use crate::app_state::AppState;
use crate::auth::{AuthControllerError, parse_basic_auth};
use crate::services::user_auth::service::UserAuthService;
use axum::extract::{Request, State};
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;
use common_infrastructure::config::{EnvironmentService, ReverseProxyConfig};
use common_infrastructure::error::ErrorSeverity::Warning;
use common_infrastructure::error::{CustomError, CustomErrorInner};
use common_infrastructure::http::get_async_sync_client;
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
use jsonwebtoken::jwk::{JwkSet, KeyAlgorithm};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use podfetch_domain::user::User;
use serde_json::Value;
use sha256::digest;
use std::collections::HashSet;
use std::sync::OnceLock;
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

static JWKS: OnceLock<JwkSet> = OnceLock::new();

pub async fn get_jwks(jwks_uri: &str) -> JwkSet {
    let client = get_async_sync_client(&ENVIRONMENT_SERVICE).build().unwrap();
    client
        .get(jwks_uri)
        .send()
        .await
        .unwrap()
        .json::<JwkSet>()
        .await
        .unwrap()
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
        AuthType::Proxy => AuthFilter::handle_proxy_auth_internal(
            req,
            &state.environment.reverse_proxy_config.clone().unwrap(),
            state.user_auth_service.as_ref(),
        ),
        AuthType::None => state.user_auth_service.ensure_admin_user(),
    }
}

fn from_key_alg_into_alg(value: KeyAlgorithm) -> Algorithm {
    match value {
        KeyAlgorithm::RS256 => Algorithm::RS256,
        KeyAlgorithm::RS384 => Algorithm::RS384,
        KeyAlgorithm::RS512 => Algorithm::RS512,
        KeyAlgorithm::ES256 => Algorithm::ES256,
        KeyAlgorithm::ES384 => Algorithm::ES384,
        KeyAlgorithm::PS256 => Algorithm::PS256,
        KeyAlgorithm::PS384 => Algorithm::PS384,
        KeyAlgorithm::PS512 => Algorithm::PS512,
        KeyAlgorithm::HS256 => Algorithm::HS256,
        KeyAlgorithm::HS384 => Algorithm::HS384,
        KeyAlgorithm::HS512 => Algorithm::HS512,
        KeyAlgorithm::EdDSA => Algorithm::EdDSA,
        KeyAlgorithm::RSA1_5 => Algorithm::ES256,
        KeyAlgorithm::RSA_OAEP => Algorithm::RS256,
        KeyAlgorithm::RSA_OAEP_256 => Algorithm::RS256,
        KeyAlgorithm::UNKNOWN_ALGORITHM => Algorithm::ES256,
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

        Ok((u.to_string(), p.to_string()))
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
        let token_res = match req.headers().get("Authorization") {
            Some(token) => match token.to_str() {
                Ok(token) => Ok(token),
                Err(_) => Err(CustomError::from(CustomErrorInner::Forbidden(Warning))),
            },
            None => Err(CustomErrorInner::UnAuthorized("Unauthorized".to_string(), Warning).into()),
        }?;

        let token = token_res.replace("Bearer ", "");

        let jwk = match JWKS.get() {
            Some(jwks) => Ok::<&JwkSet, String>(jwks),
            None => {
                let jwks = get_jwks(&environment.oidc_config.clone().unwrap().jwks_uri).await;
                JWKS.get_or_init(|| jwks);
                Ok::<&JwkSet, String>(JWKS.get().unwrap())
            }
        }
        .unwrap();

        let first_jwk = match jwk.keys.first() {
            Some(jwk) => Ok(jwk),
            None => {
                tracing::error!(
                    "No JWK found for {}",
                    environment.oidc_config.clone().unwrap().jwks_uri
                );
                Err(CustomError::from(CustomErrorInner::Forbidden(Warning)))
            }
        }?;

        let key = DecodingKey::from_jwk(first_jwk).unwrap();
        let alg = first_jwk.common.key_algorithm.unwrap();
        let mut validation = Validation::new(from_key_alg_into_alg(alg));
        let mut aud_hashset = HashSet::new();
        aud_hashset.insert(
            environment
                .oidc_config
                .clone()
                .unwrap()
                .client_id
                .to_string(),
        );
        validation.aud = Some(aud_hashset);

        let decoded_token = match decode::<Value>(&token, &key, &validation) {
            Ok(decoded) => Ok(decoded),
            Err(e) => {
                tracing::error!("Error is {e:?}");
                Err(CustomError::from(CustomErrorInner::Forbidden(Warning)))
            }
        }?;
        let username = decoded_token
            .claims
            .get("preferred_username")
            .unwrap()
            .as_str()
            .unwrap();
        let found_user = user_auth_service.find_by_username(username);

        match found_user {
            Ok(user) => Ok(user),
            Err(_) => {
                let preferred_username_claim = match decoded_token.claims.get("preferred_username")
                {
                    Some(claim) => match claim.as_str() {
                        Some(content) => Ok(content),
                        None => Err(CustomError::from(CustomErrorInner::Forbidden(Warning))),
                    },
                    None => Err(CustomErrorInner::Forbidden(Warning).into()),
                }?;

                // User is authenticated so we can onboard him if he is new
                let user = user_auth_service.create_user(
                    preferred_username_claim.to_string(),
                    "user".to_string(),
                    None,
                    false,
                )?;
                Ok(user)
            }
        }
    }

    fn handle_proxy_auth_internal(
        req: &Request,
        reverse_proxy_config: &ReverseProxyConfig,
        user_auth_service: &UserAuthService,
    ) -> Result<User, CustomError> {
        let header_val = match req
            .headers()
            .get(reverse_proxy_config.header_name.to_string())
        {
            Some(header) => Ok::<&HeaderValue, CustomError>(header),
            None => {
                info!("Reverse proxy is enabled but no header is provided");
                return Err(CustomError::from(CustomErrorInner::Forbidden(Warning)));
            }
        }?;
        let token_res = match header_val.to_str() {
            Ok(token) => Ok(token),
            Err(_) => Err(CustomError::from(CustomErrorInner::Forbidden(Warning))),
        }?;
        let found_user = user_auth_service.find_by_username(token_res);

        match found_user {
            Ok(user) => Ok(user),
            Err(_) => {
                if reverse_proxy_config.auto_sign_up {
                    let user = user_auth_service
                        .create_user(token_res.to_string(), "user".to_string(), None, false)
                        .expect("Error inserting user");
                    Ok(user)
                } else {
                    Err(CustomErrorInner::Forbidden(Warning).into())
                }
            }
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
