use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::models::user::User;
use crate::service::environment_service::ReverseProxyConfig;
use crate::utils::error::ErrorSeverity::Warning;
use crate::utils::error::{CustomError, CustomErrorInner};
use crate::utils::http_client::get_async_sync_client;
use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;
use base64::Engine;
use base64::engine::general_purpose;
use jsonwebtoken::jwk::{JwkSet, KeyAlgorithm};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use log::info;
use serde_json::Value;
use sha256::digest;
use std::collections::HashSet;
use std::sync::OnceLock;

enum AuthType {
    Basic,
    Oidc,
    Proxy,
    None,
}

pub struct AuthFilter;

pub async fn handle_basic_auth(mut request: Request, next: Next) -> Result<Response, CustomError> {
    let user = handle_auth_internal(&mut request, AuthType::Basic).await?;
    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}

static JWKS: OnceLock<JwkSet> = OnceLock::new();

pub async fn get_jwks() -> JwkSet {
    let client = get_async_sync_client().build().unwrap();
    client
        .get(&ENVIRONMENT_SERVICE.oidc_config.clone().unwrap().jwks_uri)
        .send()
        .await
        .unwrap()
        .json::<JwkSet>()
        .await
        .unwrap()
}

pub async fn handle_oidc_auth(mut request: Request, next: Next) -> Result<Response, CustomError> {
    let user = handle_auth_internal(&mut request, AuthType::Oidc).await?;
    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}

pub async fn handle_proxy_auth(mut request: Request, next: Next) -> Result<Response, CustomError> {
    let user = handle_auth_internal(&mut request, AuthType::Proxy).await?;
    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}

pub async fn handle_no_auth(mut request: Request, next: Next) -> Result<Response, CustomError> {
    let user = handle_auth_internal(&mut request, AuthType::None).await?;
    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}

async fn handle_auth_internal(req: &mut Request, auth_type: AuthType) -> Result<User, CustomError> {
    match auth_type {
        AuthType::Basic => AuthFilter::handle_basic_auth_internal(req),
        AuthType::Oidc => AuthFilter::handle_oidc_auth_internal(req).await,
        AuthType::Proxy => AuthFilter::handle_proxy_auth_internal(
            req,
            &ENVIRONMENT_SERVICE.reverse_proxy_config.clone().unwrap(),
        ),
        AuthType::None => Ok(User::create_standard_admin_user()),
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
    }
}

impl AuthFilter {
    pub fn extract_basic_auth(auth: &str) -> Result<(String, String), CustomError> {
        let auth = auth.to_string();
        let auth = auth.split(' ').collect::<Vec<&str>>();
        if auth.len() != 2 || auth[0] != "Basic" {
            return Err(CustomError::from(CustomErrorInner::Forbidden(Warning)));
        }
        let auth = auth[1];
        let auth = general_purpose::STANDARD
            .decode(auth)
            .map_err(|_| CustomError::from(CustomErrorInner::Forbidden(Warning)))?;
        let auth = String::from_utf8(auth).unwrap();
        let auth = auth.split(':').collect::<Vec<&str>>();
        let username = auth[0];
        let password = auth[1];
        Ok((username.to_string(), password.to_string()))
    }

    pub fn basic_auth_login(rq: &str) -> Result<(String, String), CustomError> {
        let (u, p) = Self::extract_basic_auth(rq)?;

        Ok((u.to_string(), p.to_string()))
    }

    fn handle_basic_auth_internal(req: &Request) -> Result<User, CustomError> {
        let opt_auth_header = req.headers().get("Authorization");
        match opt_auth_header {
            Some(header) => match header.to_str() {
                Ok(auth) => {
                    let (user, password) = AuthFilter::extract_basic_auth(auth)?;

                    let found_user = User::find_by_username(&user)
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

    async fn handle_oidc_auth_internal(req: &mut Request) -> Result<User, CustomError> {
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
                let jwks = get_jwks().await;
                JWKS.get_or_init(|| jwks);
                Ok::<&JwkSet, String>(JWKS.get().unwrap())
            }
        }
        .unwrap();

        let first_jwk = match jwk.keys.first() {
            Some(jwk) => Ok(jwk),
            None => {
                log::error!(
                    "No JWK found for {}",
                    ENVIRONMENT_SERVICE.oidc_config.clone().unwrap().jwks_uri
                );
                Err(CustomError::from(CustomErrorInner::Forbidden(Warning)))
            }
        }?;

        let key = DecodingKey::from_jwk(first_jwk).unwrap();
        let alg = first_jwk.common.key_algorithm.unwrap();
        let mut validation = Validation::new(from_key_alg_into_alg(alg));
        let mut aud_hashset = HashSet::new();
        aud_hashset.insert(
            ENVIRONMENT_SERVICE
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
                log::error!("Error is {e:?}");
                Err(CustomError::from(CustomErrorInner::Forbidden(Warning)))
            }
        }?;
        let username = decoded_token
            .claims
            .get("preferred_username")
            .unwrap()
            .as_str()
            .unwrap();
        let found_user = User::find_by_username(username);

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
                let user = User {
                    id: 0,
                    username: preferred_username_claim.to_string(),
                    role: "user".to_string(),
                    password: None,
                    explicit_consent: false,
                    created_at: chrono::Utc::now().naive_utc(),
                    api_key: None,
                }
                .insert_user()?;
                Ok(user)
            }
        }
    }

    fn handle_proxy_auth_internal(
        req: &Request,
        reverse_proxy_config: &ReverseProxyConfig,
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
        let found_user = User::find_by_username(token_res);

        match found_user {
            Ok(user) => Ok(user),
            Err(_) => {
                if reverse_proxy_config.auto_sign_up {
                    let user = User {
                        id: 0,
                        username: token_res.to_string(),
                        role: "user".to_string(),
                        password: None,
                        explicit_consent: false,
                        created_at: chrono::Utc::now().naive_utc(),
                        api_key: None,
                    }
                    .insert_user()
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
    use axum::extract::Request;

    use crate::auth_middleware::AuthFilter;

    use crate::commands::startup::tests::handle_test_startup;
    use crate::service::environment_service::ReverseProxyConfig;
    use crate::test_utils::test::create_random_user;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_basic_auth_login() {
        let result = AuthFilter::extract_basic_auth("Bearer dGVzdDp0ZXN0");
        assert!(result.is_ok());
        let (u, p) = result.unwrap();
        assert_eq!(u, "test");
        assert_eq!(p, "test");
    }

    #[test]
    #[serial]
    fn test_basic_auth_login_with_special_characters() {
        let result = AuthFilter::extract_basic_auth("Bearer dGVzdCTDvMOWOnRlc3Q=");
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
        let result = AuthFilter::handle_proxy_auth_internal(&req, &rv_config);

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
        let result = AuthFilter::handle_proxy_auth_internal(&req, &rv_config);

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
        let result = AuthFilter::handle_proxy_auth_internal(&req, &rv_config);

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
        let result = AuthFilter::handle_basic_auth_internal(&req);

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
        let result = AuthFilter::handle_basic_auth_internal(&req);

        assert!(result.is_err());
    }

    #[serial]
    #[tokio::test]
    async fn test_basic_auth_header_other_user() {
        // given
        let _router = handle_test_startup().await;
        create_random_user().insert_user().unwrap();

        let req = Request::builder()
            .header("Content-Type", "text/plain")
            .header("Authorization", "Bearer dGVzdDp0ZXN0")
            .body("".into())
            .unwrap();

        // when
        let result = AuthFilter::handle_basic_auth_internal(&req);

        // then
        assert!(result.is_err());
    }

    #[serial]
    #[tokio::test]
    async fn test_basic_auth_header_correct_user_wrong_password() {
        // given
        let _router = handle_test_startup().await;
        create_random_user().insert_user().unwrap();

        let req = Request::builder()
            .header("Content-Type", "text/plain")
            .header("Authorization", "Bearer dGVzdHVzZXI6dGVzdA==")
            .body("".into())
            .unwrap();
        // when
        let result = AuthFilter::handle_basic_auth_internal(&req);

        // then
        assert!(result.is_err());
    }
}
