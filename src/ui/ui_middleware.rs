use std::collections::HashSet;
use std::future::{ready, Future, Ready};
use std::ops::Deref;
use std::pin::Pin;
use std::rc::Rc;
use actix::fut::ok;
use actix_web::{dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform}, web, Error, HttpMessage, HttpResponse};
use actix_web::body::{EitherBody, MessageBody};
use actix_web::error::{ErrorForbidden, ErrorUnauthorized};
use actix_web::http::header;
use futures_util::future::LocalBoxFuture;
use futures_util::FutureExt;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use jsonwebtoken::jwk::Jwk;
use log::info;
use serde_json::Value;
use sha256::digest;
use crate::auth_middleware::{AuthFilter, AuthFilterMiddleware};
use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::models::user::User;

pub struct UIFilter {}

impl UIFilter {
    pub fn new() -> Self {
        UIFilter {}
    }
}

impl<S, B> Transform<S, ServiceRequest> for UIFilter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = UIFilterMiddleware<S>;
    type InitError = ();
    type Future = futures_util::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(UIFilterMiddleware { service: Rc::new(service) })
    }
}


pub struct UIFilterMiddleware<S> {
    service: Rc<S>,
}


impl<S,B> Service<ServiceRequest> for UIFilterMiddleware<S>
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
            self.handle_basic_auth(req)
        } else if ENVIRONMENT_SERVICE.oidc_configured {
            self.handle_oidc_auth(req)
        } else if ENVIRONMENT_SERVICE.reverse_proxy {
            self.handle_proxy_auth(req)
        } else {
            // It can only be no auth
            self.handle_no_auth(req)
        }
    }
}

fn create_redirect_to_login_form<B>() -> HttpResponse<EitherBody<B>> {
    let response = HttpResponse::Found()
        .insert_header((header::LOCATION, "/login"))
        .finish()
        .map_into_right_body();
    response
}

type MyFuture<B, Error> =
Pin<Box<dyn Future<Output = Result<ServiceResponse<EitherBody<B>>, Error>>>>;

impl<S, B> UIFilterMiddleware<S>
where
    B: 'static + MessageBody,
    S: 'static + Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    fn handle_basic_auth(&self, req: ServiceRequest) -> MyFuture<B, Error> {
        let opt_auth_header = req.headers().get("Authorization");

        match opt_auth_header {
            Some(header) => match header.to_str() {
                Ok(auth) => {
                    let (username, password) = AuthFilter::extract_basic_auth(auth);
                    let found_user = User::find_by_username(username.as_str());

                    if found_user.is_err() {
                        return Box::pin(async move {
                            Ok(ServiceResponse::new(req.request().clone(),
                                                    create_redirect_to_login_form()))
                        });
                    }
                    let unwrapped_user = found_user.unwrap();

                    if let Some(admin_username) = ENVIRONMENT_SERVICE.username.clone() {
                        if unwrapped_user.username.clone() == admin_username {
                            return match ENVIRONMENT_SERVICE.password.is_some()
                                && digest(password) == ENVIRONMENT_SERVICE.password.clone().unwrap()
                            {
                                true => {
                                    req.extensions_mut().insert(unwrapped_user);
                                    let service = Rc::clone(&self.service);
                                    async move {
                                        service.call(req).await.map(|res| res.map_into_left_body())
                                    }
                                        .boxed_local()
                                }
                                false => Box::pin(async move {
                                    Ok(ServiceResponse::new(req.request()
                                                                .clone(),
                                                            create_redirect_to_login_form()))
                                })
                            };
                        }
                    }

                    if unwrapped_user.password.clone().unwrap() == digest(password) {
                        req.extensions_mut().insert(unwrapped_user);
                        let service = Rc::clone(&self.service);
                        async move { service.call(req).await.map(|res| res.map_into_left_body()) }
                            .boxed_local()
                    } else {
                        Box::pin(async move {
                            Ok(ServiceResponse::new(req.request().clone(),
                                                    create_redirect_to_login_form()))
                        })
                    }
                }
                Err(_) => Box::pin(async move {
                    Ok(ServiceResponse::new(req.request().clone(),
                                            create_redirect_to_login_form()))
                }),
            },
            None => Box::pin(async move {
                Ok(ServiceResponse::new(req.request().clone(),
                                        create_redirect_to_login_form()))
            }),
        }
    }

    fn handle_oidc_auth(&self, req: ServiceRequest) -> MyFuture<B, Error> {

        let token_res = match req.headers().get("Authorization"){
            Some(header) => {
                match header.to_str() {
                    Ok(auth) => {
                        auth.to_string()
                    }
                    Err(_) => {
                        return Box::pin(async move {
                            Ok(ServiceResponse::new(req.request().clone(),
                                                    create_redirect_to_login_form()))
                        })
                    }
                }}
                None => {
                return Box::pin(async move {
                    Ok(ServiceResponse::new(req.request().clone(),
                                            create_redirect_to_login_form()))
                })
            }
        };
        let token = token_res.replace("Bearer ", "");

        let jwk = req.app_data::<web::Data<Option<Jwk>>>().cloned().unwrap();

        // Create a DecodingKey from a PEM-encoded RSA string

        let key = DecodingKey::from_jwk(&jwk.as_ref().clone().unwrap()).unwrap();
        let mut validation = Validation::new(Algorithm::RS256);
        validation.aud = Some(
            req.app_data::<web::Data<HashSet<String>>>()
                .unwrap()
                .clone()
                .into_inner()
                .deref()
                .clone(),
        );

        match decode::<Value>(&token, &key, &validation) {
            Ok(decoded) => {
                let username = decoded
                    .claims
                    .get("preferred_username")
                    .unwrap()
                    .as_str()
                    .unwrap();
                let found_user = User::find_by_username(username);
                let service = Rc::clone(&self.service);

                match found_user {
                    Ok(user) => {
                        req.extensions_mut().insert(user);
                        async move { service.call(req).await.map(|res| res.map_into_left_body()) }
                            .boxed_local()
                    }
                    Err(_) => {
                        // User is authenticated so we can onboard him if he is new
                        let user = User::insert_user(&mut User {
                            id: 0,
                            username: decoded
                                .claims
                                .get("preferred_username")
                                .unwrap()
                                .as_str()
                                .unwrap()
                                .to_string(),
                            role: "user".to_string(),
                            password: None,
                            explicit_consent: false,
                            created_at: chrono::Utc::now().naive_utc(),
                            api_key: None,
                        })
                            .expect("Error inserting user");
                        req.extensions_mut().insert(user);
                        async move { service.call(req).await.map(|res| res.map_into_left_body()) }
                            .boxed_local()
                    }
                }
            }
            Err(e) => {
                info!("Error decoding token: {:?}", e);
                Box::pin(async move {
                    Ok(ServiceResponse::new(req.request().clone(),
                                            create_redirect_to_login_form()))
                })
            }
        }
    }

    fn handle_no_auth(&self, req: ServiceRequest) -> MyFuture<B, Error> {
        let user = User::create_standard_admin_user();
        req.extensions_mut().insert(user);
        let service = Rc::clone(&self.service);
        async move { service.call(req).await.map(|res| res.map_into_left_body()) }.boxed_local()
    }

    fn handle_proxy_auth(&self, req: ServiceRequest) -> MyFuture<B, Error> {
        let config = ENVIRONMENT_SERVICE
            .reverse_proxy_config
            .clone()
            .unwrap();

        let header_val = req.headers().get(config.header_name);

        if let Some(header_val) = header_val {
            let token_res = header_val.to_str();
            return match token_res {
                Ok(token) => {
                    let found_user = User::find_by_username(token);
                    let service = Rc::clone(&self.service);

                    return match found_user {
                        Ok(user) => {
                            req.extensions_mut().insert(user);
                            return async move {
                                service.call(req).await.map(|res| res.map_into_left_body())
                            }
                                .boxed_local();
                        }
                        Err(_) => {
                            if config.auto_sign_up {
                                let user = User::insert_user(&mut User {
                                    id: 0,
                                    username: token.to_string(),
                                    role: "user".to_string(),
                                    password: None,
                                    explicit_consent: false,
                                    created_at: chrono::Utc::now().naive_utc(),
                                    api_key: None,
                                })
                                    .expect("Error inserting user");
                                req.extensions_mut().insert(user);
                                return async move {
                                    service.call(req).await.map(|res| res.map_into_left_body())
                                }
                                    .boxed_local();
                            } else {
                                Box::pin(async move {
                                    Ok(ServiceResponse::new(req.request().clone(),
                                                            create_redirect_to_login_form()))
                                })
                            }
                        }
                    };
                }
                Err(_) => Box::pin(async move {
                    Ok(ServiceResponse::new(req.request().clone(),
                                            create_redirect_to_login_form()))
                })
            };
        }
        Box::pin(async move {
            Ok(ServiceResponse::new(req.request().clone(),
                                    create_redirect_to_login_form()))
        })
    }
}
