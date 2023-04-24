use actix_session::SessionMiddleware;
use actix_session::storage::CookieSessionStore;
use actix_web::{Either, Error, Handler, HttpRequest, HttpResponse, Scope, web};
use actix_web::body::{BoxBody, EitherBody};
use actix_web::dev::{Service, ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::error::ErrorUnauthorized;
use actix_web::http::header::HeaderMap;
use actix_web_httpauth::middleware::HttpAuthentication;
use awc::cookie::Key;
use futures::TryFutureExt;
use futures_util::future::LocalBoxFuture;
use futures_util::FutureExt;
use serde_json::json;
use utoipa::openapi::security::Scopes;
use crate::config::dbconfig::establish_connection;
use crate::gpodder::device::device_controller::{get_devices_of_user, post_device};
use crate::{DbPool, extract_basic_auth, validator};
use crate::constants::constants::ERROR_LOGIN_MESSAGE;
use crate::gpodder::auth::auth::login;
use crate::gpodder::parametrization::get_client_parametrization;

pub fn get_gpodder_api(pool: DbPool) ->Scope<impl ServiceFactory<ServiceRequest, Config =
(), Response = ServiceResponse, Error = Error, InitError = ()>>{
    let secret_key = Key::generate();

    web::scope("/api/2")
        .wrap(SessionMiddleware::new(CookieSessionStore::default(),secret_key))
        .service(login)
        .service(get_authenticated_api(pool.clone()))

}


pub fn get_authenticated_api(pool: DbPool) ->actix_web::Scope<impl ServiceFactory<ServiceRequest, Config = (), Response = ServiceResponse, Error = actix_web::Error, InitError = ()>>{
    web::scope("")
        .wrap_fn(|rq, srv|{
            let srv1  = srv;
            let res = rq.cookie("sessionid").clone();
            async move {
                if res.is_none(){
                let mut resp = rq.into_response(HttpResponse::Unauthorized().finish());
                return Ok(resp);
            }
                let fut = srv1.call(rq).await;

            Ok(fut.unwrap())
            }
        })
        .service(post_device)
        .service(get_devices_of_user)
}


