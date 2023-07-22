use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use actix::fut::{ok};
use futures_util::FutureExt;
use actix_web::{dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform}, Error, HttpMessage, web};
use actix_web::body::{EitherBody, MessageBody};
use actix_web::error::{ErrorForbidden, ErrorUnauthorized};
use base64::Engine;
use base64::engine::general_purpose;
use futures_util::future::{LocalBoxFuture, Ready};
use dotenv::var;
use jsonwebtoken::{Algorithm, decode, DecodingKey, Validation};
use jsonwebtoken::jwk::Jwk;
use log::info;
use serde_json::{from_str, Value};
use crate::constants::inner_constants::{BASIC_AUTH, OIDC_AUTH, PASSWORD, USERNAME};
use crate::{DbPool};
use crate::models::user::User;
use sha256::digest;
use crate::models::oidc_model::{CustomJwk, CustomJwkSet};
use crate::mutex::LockResultExt;
use crate::service::jwkservice::JWKService;

pub struct AuthFilter {
}

impl AuthFilter {
    pub fn new() -> Self {
        AuthFilter {
        }
    }
}

#[derive(Default)]
pub struct AuthFilterMiddleware<S>{
    service: Rc<S>
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
            service: Rc::new(service)
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
        return if var(BASIC_AUTH).is_ok() {
            self.handle_basic_auth(req)
        } else if var(OIDC_AUTH).is_ok() {
            self.handle_oidc_auth(req)
        } else {
            // It can only be no auth
            self.handle_no_auth(req)
        }
    }
}

type MyFuture<B, Error> = Pin<Box<dyn Future<Output = Result<ServiceResponse<EitherBody<B>>, Error>>>>;

impl<S, B> AuthFilterMiddleware<S> where B: 'static + MessageBody, S: 'static + Service<ServiceRequest, Response=ServiceResponse<B>, Error=Error>, S::Future: 'static {
    fn handle_basic_auth(&self, req: ServiceRequest) -> MyFuture<B, Error> {
        let opt_auth_header = req.headers().get("Authorization");
        if opt_auth_header.is_none() {
            return Box::pin(ok(req.error_response(ErrorUnauthorized("Unauthorized")).map_into_right_body()));
        }
        let authorization = opt_auth_header.unwrap().to_str();
        return match authorization {
            Ok(auth) => {
                let (username, password) = AuthFilter::extract_basic_auth(auth);
                let res = req.app_data::<web::Data<DbPool>>().unwrap();
                let found_user = User::find_by_username(username.as_str(), &mut res.get().unwrap());

                if found_user.is_err() {
                    return Box::pin(ok(req.error_response(ErrorUnauthorized("Unauthorized"))
                        .map_into_right_body()))
                }
                let unwrapped_user = found_user.unwrap();

                if unwrapped_user.clone().username == var(USERNAME).unwrap(){
                    return match password == var(PASSWORD).unwrap() {
                        true => {
                            req.extensions_mut().insert(unwrapped_user);
                            let service = Rc::clone(&self.service);
                            async move {
                                service
                                    .call(req)
                                    .await
                                    .map(|res| res.map_into_left_body())
                            }
                                .boxed_local()
                        },
                        false => {
                            Box::pin(ok(req.error_response(ErrorUnauthorized("Unauthorized"))
                                .map_into_right_body()))
                        }
                    }
                }

                if unwrapped_user.password.clone().unwrap() == digest(password) {
                    req.extensions_mut().insert(unwrapped_user);
                    let service = Rc::clone(&self.service);
                    async move {
                        service
                            .call(req)
                            .await
                            .map(|res| res.map_into_left_body())
                    }
                        .boxed_local()
                } else {
                    Box::pin(ok(req.error_response(ErrorUnauthorized("Unauthorized")).map_into_right_body()))
                }
            },
            Err(_) => {
                Box::pin(ok(req.error_response(ErrorUnauthorized("Unauthorized")).map_into_right_body()))
            }
        }
    }

    fn handle_oidc_auth(&self, req: ServiceRequest) -> MyFuture<B, Error> {
        let token_res = req.headers().get("Authorization").unwrap().to_str();
        if token_res.is_err() {
            return Box::pin(ok(req.error_response(ErrorUnauthorized("Unauthorized")).map_into_right_body()));
        }
        let token = token_res.unwrap().replace("Bearer ", "");

        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards").as_secs();

        let response:CustomJwkSet;
        let binding = req.app_data::<web::Data<Mutex<JWKService>>>().cloned().unwrap();
        let mut jwk_service = binding.lock()
            .ignore_poison();
        match jwk_service.clone().jwk {
            Some(jwk)=>{
                if since_the_epoch-jwk_service.timestamp>3600{
                    //refetch and update timestamp
                    info!("Renewing jwk set");
                    response = AuthFilter::get_jwk();
                    jwk_service.jwk = Some(response.clone());
                    jwk_service.timestamp = since_the_epoch
                }
                else{
                    info!("Using cached jwk set");
                    response = jwk;
                }
            }
            None=>{
                // Fetch on cold start
                response = AuthFilter::get_jwk();
                jwk_service.jwk = Some(response.clone());
                jwk_service.timestamp = since_the_epoch
            }
        }

        // Filter out all unknown algorithms
        let response = response.clone().keys.into_iter().filter(|x| {
            x.alg.eq(&"RS256")
        }).collect::<Vec<CustomJwk>>();

        let jwk = response.clone();
        let custom_jwk = jwk.get(0).expect("Your jwk set needs to have RS256");

        let jwk_string = serde_json::to_string(&custom_jwk).unwrap();

        let jwk = from_str::<Jwk>(&jwk_string).unwrap();
        let key = DecodingKey::from_jwk(&jwk).unwrap();
        let validation = Validation::new(Algorithm::RS256);
        return match decode::<Value>(&token, &key, &validation) {
            Ok(decoded) => {
                let username = decoded.claims.get("preferred_username").unwrap().as_str().unwrap();
                let pool = req.app_data::<web::Data<DbPool>>().cloned().unwrap();
                let found_user = User::find_by_username(username, &mut pool.get().unwrap());
                let service = Rc::clone(&self.service);

                match found_user {
                    Ok(user) => {
                        req.extensions_mut().insert(user);
                        async move {
                            service
                                .call(req)
                                .await
                                .map(|res| res.map_into_left_body())
                        }
                            .boxed_local()
                    },
                    Err(_) => {
                        // User is authenticated so we can onboard him if he is new
                        let user = User::insert_user(&mut User {
                            id: 0,
                            username: decoded.claims.get("preferred_username").unwrap().as_str().unwrap().to_string(),
                            role: "user".to_string(),
                            password: None,
                            explicit_consent: false,
                            created_at: chrono::Utc::now().naive_utc()
                        }, &mut pool.get().unwrap()).expect("Error inserting user");
                        req.extensions_mut().insert(user);
                        async move {
                            service
                                .call(req)
                                .await
                                .map(|res| res.map_into_left_body())
                        }
                            .boxed_local()
                    }
                }
            },
            _ => {
                Box::pin(ok(req.error_response(ErrorForbidden("Forbidden"))
                    .map_into_right_body()))
            }
        }
    }

    fn handle_no_auth(&self, req: ServiceRequest) -> MyFuture<B, Error> {
        let user = User::create_standard_admin_user();
        req.extensions_mut().insert(user);
        let service = Rc::clone(&self.service);
        async move {
            service
                .call(req)
                .await
                .map(|res| res.map_into_left_body())
        }
            .boxed_local()
    }
}

impl AuthFilter{
    pub fn extract_basic_auth(auth: &str) -> (String, String) {
        let auth = auth.to_string();
        let auth = auth.split(' ').collect::<Vec<&str>>();
        let auth = auth[1];
        let auth = general_purpose::STANDARD.decode(auth).unwrap();
        let auth = String::from_utf8(auth).unwrap();
        let auth = auth.split(':').collect::<Vec<&str>>();
        let username = auth[0];
        let password = auth[1];
        (username.to_string(), password.to_string())
    }

    pub fn get_jwk() -> CustomJwkSet {
        let jwk_uri = var("OIDC_JWKS").expect("OIDC_JWKS must be set");
         reqwest::blocking::get(jwk_uri).unwrap()
            .json::<CustomJwkSet>().unwrap()
    }

    pub fn basic_auth_login(rq: String) -> (String, String) {
        let (u,p) = Self::extract_basic_auth(rq.as_str());

        (u.to_string(),p.to_string())
    }
}