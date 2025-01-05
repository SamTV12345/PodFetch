use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::models::user::User;
use actix::fut::ok;
use actix_web::body::{EitherBody, MessageBody};
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web, Error, HttpMessage,
};
use base64::engine::general_purpose;
use base64::Engine;
use std::collections::HashSet;
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::rc::Rc;

use crate::service::environment_service::ReverseProxyConfig;
use crate::utils::error::CustomError;
use futures_util::future::{LocalBoxFuture, Ready};
use futures_util::FutureExt;
use jsonwebtoken::jwk::Jwk;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use log::info;
use serde_json::Value;
use sha256::digest;

pub struct AuthFilter {}

impl AuthFilter {
    pub fn new() -> Self {
        AuthFilter {}
    }
}

#[derive(Default)]
pub struct AuthFilterMiddleware<S> {
    service: Rc<S>,
}

enum AuthType {
    Basic,
    Oidc,
    Proxy,
    None,
}

impl<S, B> Transform<S, ServiceRequest> for AuthFilter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = AuthFilterMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthFilterMiddleware {
            service: Rc::new(service),
        })
    }
}

impl<S, B> Service<ServiceRequest> for AuthFilterMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
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

type MyFuture<B, Error> =
    Pin<Box<dyn Future<Output = Result<ServiceResponse<EitherBody<B>>, Error>>>>;

impl<S, B> AuthFilterMiddleware<S>
where
    B: 'static + MessageBody,
    S: 'static + Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    fn handle_auth(&self, req: ServiceRequest, auth_type: AuthType) -> MyFuture<B, Error> {
        let result = match auth_type {
            AuthType::Basic => AuthFilter::handle_basic_auth_internal(&req),
            AuthType::Oidc => AuthFilter::handle_oidc_auth_internal(&req),
            AuthType::Proxy => AuthFilter::handle_proxy_auth_internal(
                &req,
                &ENVIRONMENT_SERVICE.reverse_proxy_config.clone().unwrap(),
            ),
            AuthType::None => Ok(User::create_standard_admin_user()),
        };
        match result {
            Ok(user) => {
                req.extensions_mut().insert(user);
                let service = Rc::clone(&self.service);
                async move { service.call(req).await.map(|res| res.map_into_left_body()) }
                    .boxed_local()
            }
            Err(e) => Box::pin(ok(req.error_response(e).map_into_right_body())),
        }
    }
}

impl AuthFilter {
    pub fn extract_basic_auth(auth: &str) -> Result<(String, String), CustomError> {
        let auth = auth.to_string();
        let auth = auth.split(' ').collect::<Vec<&str>>();
        let auth = auth[1];
        let auth = general_purpose::STANDARD
            .decode(auth)
            .map_err(|_| CustomError::Forbidden)?;
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

    fn handle_basic_auth_internal(req: &ServiceRequest) -> Result<User, CustomError> {
        let opt_auth_header = req.headers().get("Authorization");
        match opt_auth_header {
            Some(header) => match header.to_str() {
                Ok(auth) => {
                    let (user, password) = AuthFilter::extract_basic_auth(auth)?;

                    let found_user =
                        User::find_by_username(&user).map_err(|_| CustomError::Forbidden)?;

                    if let Some(admin_username) = &ENVIRONMENT_SERVICE.username {
                        if &found_user.username == admin_username {
                            return if let Some(env_password) = &ENVIRONMENT_SERVICE.password {
                                if &digest(password) == env_password {
                                    Ok(found_user)
                                } else {
                                    Err(CustomError::Forbidden)
                                }
                            } else {
                                Err(CustomError::Forbidden)
                            };
                        }
                    }

                    if let Some(password_from_user) = &found_user.password {
                        if password_from_user == &digest(password) {
                            Ok(found_user)
                        } else {
                            Err(CustomError::Forbidden)
                        }
                    } else {
                        Err(CustomError::Forbidden)
                    }
                }
                Err(_) => Err(CustomError::Forbidden),
            },
            None => Err(CustomError::Forbidden),
        }
    }

    fn handle_oidc_auth_internal(req: &ServiceRequest) -> Result<User, CustomError> {
        let token_res = match req.headers().get("Authorization") {
            Some(token) => match token.to_str() {
                Ok(token) => Ok(token),
                Err(_) => Err(CustomError::Forbidden),
            },
            None => Err(CustomError::UnAuthorized("Unauthorized".to_string())),
        }?;

        let token = token_res.replace("Bearer ", "");
        let jwk = match req.app_data::<web::Data<Option<Jwk>>>() {
            Some(jwk) => match jwk.get_ref() {
                Some(jwk) => Ok(jwk),
                None => Err(CustomError::Forbidden),
            },
            None => Err(CustomError::Forbidden),
        }?;
        let key = DecodingKey::from_jwk(jwk).unwrap();
        let mut validation = Validation::new(Algorithm::RS256);
        validation.aud = Some(
            req.app_data::<web::Data<HashSet<String>>>()
                .unwrap()
                .clone()
                .into_inner()
                .deref()
                .clone(),
        );

        let decoded_token = match decode::<Value>(&token, &key, &validation) {
            Ok(decoded) => Ok(decoded),
            Err(_) => Err(CustomError::Forbidden),
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
                        None => Err(CustomError::Forbidden),
                    },
                    None => Err(CustomError::Forbidden),
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
        req: &ServiceRequest,
        reverse_proxy_config: &ReverseProxyConfig,
    ) -> Result<User, CustomError> {
        let header_val = match req
            .headers()
            .get(reverse_proxy_config.header_name.to_string())
        {
            Some(header) => Ok(header),
            None => {
                info!("Reverse proxy is enabled but no header is provided");
                return Err(CustomError::Forbidden);
            }
        }?;
        let token_res = match header_val.to_str() {
            Ok(token) => Ok(token),
            Err(_) => Err(CustomError::Forbidden),
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
                    Err(CustomError::Forbidden)
                }
            }
        }
    }
}

#[cfg(test)]
mod test {

    use actix_web::http::header::ContentType;
    use actix_web::test;

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

    #[actix_web::test]
    #[serial]
    async fn test_proxy_auth_with_no_header_in_request_auto_sign_up() {
        clear_users();
        let rv_config = ReverseProxyConfig {
            header_name: "X-Auth".to_string(),
            auto_sign_up: true,
        };

        let req = test::TestRequest::default()
            .insert_header(ContentType::plaintext())
            .to_srv_request();
        let result = AuthFilter::handle_proxy_auth_internal(&req, &rv_config);

        assert!(result.is_err());
    }

    #[actix_web::test]
    #[serial]
    async fn test_proxy_auth_with_header_in_request_auto_sign_up() {
        clear_users();
        let rv_config = ReverseProxyConfig {
            header_name: "X-Auth".to_string(),
            auto_sign_up: false,
        };

        let req = test::TestRequest::default()
            .insert_header(ContentType::plaintext())
            .insert_header(("X-Auth", "test"))
            .to_srv_request();
        let result = AuthFilter::handle_proxy_auth_internal(&req, &rv_config);

        assert!(result.is_err());
    }

    #[actix_web::test]
    #[serial]
    async fn test_proxy_auth_with_header_in_request_auto_sign_up_false() {
        clear_users();
        let rv_config = ReverseProxyConfig {
            header_name: "X-Auth".to_string(),
            auto_sign_up: true,
        };

        let req = test::TestRequest::default()
            .insert_header(ContentType::plaintext())
            .insert_header(("X-Auth", "test"))
            .to_srv_request();
        let result = AuthFilter::handle_proxy_auth_internal(&req, &rv_config);

        assert!(result.is_ok());
    }

    #[actix_web::test]
    #[serial]
    async fn test_basic_auth_no_header() {
        clear_users();

        let req = test::TestRequest::default()
            .insert_header(ContentType::plaintext())
            .to_srv_request();
        let result = AuthFilter::handle_basic_auth_internal(&req);

        assert!(result.is_err());
    }

    #[actix_web::test]
    #[serial]
    async fn test_basic_auth_header_no_user() {
        clear_users();

        let req = test::TestRequest::default()
            .insert_header(ContentType::plaintext())
            .insert_header(("Authorization", "Bearer dGVzdDp0ZXN0"))
            .to_srv_request();
        let result = AuthFilter::handle_basic_auth_internal(&req);

        assert!(result.is_err());
    }

    #[actix_web::test]
    #[serial]
    async fn test_basic_auth_header_other_user() {
        // given
        clear_users();
        create_random_user().insert_user().unwrap();

        let req = test::TestRequest::default()
            .insert_header(ContentType::plaintext())
            .insert_header(("Authorization", "Bearer dGVzdDp0ZXN0"))
            .to_srv_request();
        // when
        let result = AuthFilter::handle_basic_auth_internal(&req);

        // then
        assert!(result.is_err());
    }

    #[actix_web::test]
    #[serial]
    async fn test_basic_auth_header_correct_user_wrong_password() {
        // given
        clear_users();
        create_random_user().insert_user().unwrap();

        let req = test::TestRequest::default()
            .insert_header(ContentType::plaintext())
            .insert_header(("Authorization", "Bearer dGVzdHVzZXI6dGVzdA=="))
            .to_srv_request();
        // when
        let result = AuthFilter::handle_basic_auth_internal(&req);

        // then
        assert!(result.is_err());
    }
}
