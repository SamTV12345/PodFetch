use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::models::user::User;
use base64::engine::general_purpose;
use base64::Engine;
use std::collections::HashSet;
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};
use axum::extract::{Request, State};
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::RequestExt;
use axum::response::{IntoResponse, Response};
use crate::service::environment_service::ReverseProxyConfig;
use crate::utils::error::{CustomError, CustomErrorInner};
use futures_util::future::{BoxFuture, LocalBoxFuture, Ready};
use futures_util::FutureExt;
use jsonwebtoken::jwk::Jwk;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use log::info;
use serde_json::Value;
use sha256::digest;
use tower::{Layer, Service};

pub struct AuthFilter {}

impl <S> Layer<S> for AuthFilter {
    type Service = AuthFilterMiddleware<S>;

    fn layer(&self, service: S) -> Self::Service {
        AuthFilterMiddleware {
            inner: Rc::new(service),
        }
    }
}

#[derive(Default)]
pub struct AuthFilterMiddleware<S> {
    inner: Rc<S>,
}

enum AuthType {
    Basic,
    Oidc,
    Proxy,
    None,
}

impl<S> Service<Request> for AuthFilterMiddleware<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    // `BoxFuture` is a type alias for `Pin<Box<dyn Future + Send + 'a>>`
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        if ENVIRONMENT_SERVICE.http_basic {
            self.handle_auth(req, AuthType::Basic)
        } else if ENVIRONMENT_SERVICE.oidc_configured {
            self.handle_auth(req, AuthType::Oidc)
        } else if ENVIRONMENT_SERVICE.reverse_proxy {
            self.handle_auth(req, AuthType::Proxy)
        } else {
            // It can only be no auth
            self.handle_auth(req, AuthType::None)
        }
    }
}


pub async fn handle_auth(
    State(jwk): State<Option<Jwk>>,
    State(audience): State<HashSet<String>>,
    mut request: Request,
    next: Next) -> Result<impl IntoResponse, CustomError> {
    if ENVIRONMENT_SERVICE.http_basic {
        let user = handle_auth_internal(&mut request, AuthType::Basic, jwk, audience)?;
        request.extensions_mut().insert(user);
        Ok(next.run(request).await)
    } else if ENVIRONMENT_SERVICE.oidc_configured {
        let user = handle_auth_internal(&mut request, AuthType::Oidc, jwk, audience)?;
        request.extensions_mut().insert(user);
        Ok(next.run(request).await)
    } else if ENVIRONMENT_SERVICE.reverse_proxy {
        let user = handle_auth_internal(&mut request, AuthType::Proxy, jwk, audience)?;
        request.extensions_mut().insert(user);
        Ok(next.run(request).await)
    } else {
        // It can only be no auth
        let user = handle_auth_internal(&mut request, AuthType::None, jwk, audience)?;
        request.extensions_mut().insert(user);
        Ok(next.run(request).await)
    }
}


fn handle_auth_internal(req: &mut Request, auth_type: AuthType, jwk: Option<Jwk>, audience:
HashSet<String>) -> Result<User, CustomError> {
    match auth_type {
        AuthType::Basic => AuthFilter::handle_basic_auth_internal(&req),
        AuthType::Oidc => AuthFilter::handle_oidc_auth_internal(req, jwk, audience),
        AuthType::Proxy => AuthFilter::handle_proxy_auth_internal(
            &req,
            &ENVIRONMENT_SERVICE.reverse_proxy_config.clone().unwrap(),
        ),
        AuthType::None => Ok(User::create_standard_admin_user()),
    }
}


impl AuthFilter {
    pub fn extract_basic_auth(auth: &str) -> Result<(String, String), CustomError> {
        let auth = auth.to_string();
        let auth = auth.split(' ').collect::<Vec<&str>>();
        let auth = auth[1];
        let auth = general_purpose::STANDARD
            .decode(auth)
            .map_err(|_| CustomError::from(CustomErrorInner::Forbidden))?;
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

                    let found_user =
                        User::find_by_username(&user).map_err(|_| CustomErrorInner::Forbidden)?;

                    if let Some(admin_username) = &ENVIRONMENT_SERVICE.username {
                        if &found_user.username == admin_username {
                            return if let Some(env_password) = &ENVIRONMENT_SERVICE.password {
                                if &digest(password) == env_password {
                                    Ok(found_user)
                                } else {
                                    Err(CustomErrorInner::Forbidden.into())
                                }
                            } else {
                                Err(CustomErrorInner::Forbidden.into())
                            };
                        }
                    }

                    if let Some(password_from_user) = &found_user.password {
                        if password_from_user == &digest(password) {
                            Ok(found_user)
                        } else {
                            Err(CustomErrorInner::Forbidden.into())
                        }
                    } else {
                        Err(CustomErrorInner::Forbidden.into())
                    }
                }
                Err(_) => Err(CustomErrorInner::Forbidden.into()),
            },
            None => Err(CustomErrorInner::Forbidden.into()),
        }
    }

    fn handle_oidc_auth_internal(req: &mut Request, jwk: Option<Jwk>, audience: HashSet<String>) ->
                                                                                     Result<User, CustomError> {
        let token_res = match req.headers().get("Authorization") {
            Some(token) => match token.to_str() {
                Ok(token) => Ok(token),
                Err(_) => Err(CustomError::from(CustomErrorInner::Forbidden)),
            },
            None => Err(CustomErrorInner::UnAuthorized("Unauthorized".to_string()).into()),
        }?;

        let token = token_res.replace("Bearer ", "");
        let jwk = match jwk {
            Some(jwk) => match jwk {
                Some(jwk) => Ok(jwk),
                None => Err(CustomError::from(CustomErrorInner::Forbidden)),
            },
            None => Err(CustomError::from(CustomErrorInner::Forbidden)),
        }?;
        let key = DecodingKey::from_jwk(jwk).unwrap();
        let mut validation = Validation::new(Algorithm::RS256);
        validation.aud = Some(
            audience
        );

        let decoded_token = match decode::<Value>(&token, &key, &validation) {
            Ok(decoded) => Ok(decoded),
            Err(_) => Err(CustomError::from(CustomErrorInner::Forbidden)),
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
                        None => Err(CustomError::from(CustomErrorInner::Forbidden)),
                    },
                    None => Err(CustomErrorInner::Forbidden.into()),
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
                return Err(CustomError::from(CustomErrorInner::Forbidden));
            }
        }?;
        let token_res = match header_val.to_str() {
            Ok(token) => Ok(token),
            Err(_) => Err(CustomError::from(CustomErrorInner::Forbidden)),
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
                    Err(CustomErrorInner::Forbidden.into())
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use axum::extract::Request;
    use axum::Router;
    use axum_test::TestRequest;
    use serial_test::serial;

    use crate::auth_middleware::AuthFilter;

    use crate::service::environment_service::ReverseProxyConfig;
    use crate::test_utils::test::{clear_users, create_random_user};

    #[test]
    async fn test_basic_auth_login() {
        let result = AuthFilter::extract_basic_auth("Bearer dGVzdDp0ZXN0");
        assert!(result.is_ok());
        let (u, p) = result.unwrap();
        assert_eq!(u, "test");
        assert_eq!(p, "test");
    }

    #[test]
    async fn test_basic_auth_login_with_special_characters() {
        let result = AuthFilter::extract_basic_auth("Bearer dGVzdCTDvMOWOnRlc3Q=");
        assert!(result.is_ok());
        let (u, p) = result.unwrap();
        assert_eq!(u, "test$üÖ");
        assert_eq!(p, "test");
    }

    #[serial]
    #[test]
    async fn test_proxy_auth_with_no_header_in_request_auto_sign_up() {
        clear_users();
        let rv_config = ReverseProxyConfig {
            header_name: "X-Auth".to_string(),
            auto_sign_up: true,
        };

        let req = Request::builder().header("Content-Type", "text/plain").body("".into()).unwrap();
        let result = AuthFilter::handle_proxy_auth_internal(&req, &rv_config);

        assert!(result.is_err());
    }

    #[serial]
    #[test]
    async fn test_proxy_auth_with_header_in_request_auto_sign_up() {
        clear_users();
        let rv_config = ReverseProxyConfig {
            header_name: "X-Auth".to_string(),
            auto_sign_up: false,
        };
        let req = Request::builder().header("Content-Type", "text/plain")
            .header("X-Auth", "test")
            .body("".into()).unwrap();
        let result = AuthFilter::handle_proxy_auth_internal(&req, &rv_config);

        assert!(result.is_err());
    }

    #[serial]
    #[test]
    async fn test_proxy_auth_with_header_in_request_auto_sign_up_false() {
        clear_users();
        let rv_config = ReverseProxyConfig {
            header_name: "X-Auth".to_string(),
            auto_sign_up: true,
        };

        let req = Request::builder().header("Content-Type", "text/plain")
            .header("X-Auth", "test")
            .body("".into()).unwrap();
        let result = AuthFilter::handle_proxy_auth_internal(&req, &rv_config);

        assert!(result.is_ok());
    }

    #[serial]
    #[test]
    async fn test_basic_auth_no_header() {
        clear_users();

        let req = Request::builder().header("Content-Type", "text/plain")
            .body("".into()).unwrap();
        let result = AuthFilter::handle_basic_auth_internal(&req);

        assert!(result.is_err());
    }

    #[serial]
    #[test]
    async fn test_basic_auth_header_no_user() {
        clear_users();

        let req = Request::builder().header("Content-Type", "text/plain")
            .header("Authorization", "Bearer dGVzdDp0ZXN0")
            .body("".into()).unwrap();
        let result = AuthFilter::handle_basic_auth_internal(&req);

        assert!(result.is_err());
    }

    #[serial]
    #[test]
    async fn test_basic_auth_header_other_user() {
        // given
        clear_users();
        create_random_user().insert_user().unwrap();

        let req = Request::builder().header("Content-Type", "text/plain")
            .header("Authorization", "Bearer dGVzdDp0ZXN0")
            .body("".into()).unwrap();

        // when
        let result = AuthFilter::handle_basic_auth_internal(&req);

        // then
        assert!(result.is_err());
    }

    #[serial]
    #[test]
    async fn test_basic_auth_header_correct_user_wrong_password() {
        // given
        clear_users();
        create_random_user().insert_user().unwrap();

        let req = Request::builder().header("Content-Type", "text/plain")
            .header("Authorization", "Bearer dGVzdHVzZXI6dGVzdA==")
            .body("".into()).unwrap();
        // when
        let result = AuthFilter::handle_basic_auth_internal(&req);

        // then
        assert!(result.is_err());
    }
}
