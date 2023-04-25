use actix_session::SessionMiddleware;
use actix_session::storage::CookieSessionStore;
use actix_web::{Error, Scope, web};
use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use awc::cookie::Key;
use crate::gpodder::device::device_controller::{get_devices_of_user, post_device};
use crate::{DbPool};
use crate::gpodder::auth::auth::login;
use crate::gpodder::episodes::episodes::{get_episode_actions, upload_episode_actions};
use crate::gpodder::subscription::subscriptions::{get_subscriptions, upload_subscription_changes};

pub fn get_gpodder_api(pool: DbPool) ->Scope<impl ServiceFactory<ServiceRequest, Config =
(), Response = ServiceResponse, Error = Error, InitError = ()>>{
    let secret_key = Key::generate();

    web::scope("/api/2")
        .wrap(SessionMiddleware::new(CookieSessionStore::default(),secret_key))
        .service(login)
        .service(get_authenticated_api(pool.clone()))

}


pub fn get_authenticated_api(_: DbPool) ->Scope<impl ServiceFactory<ServiceRequest,
    Config = (), Response = ServiceResponse, Error = Error, InitError = ()>>{
    web::scope("")
        .service(post_device)
        .service(get_devices_of_user)
        .service(get_subscriptions)
        .service(upload_subscription_changes)
        .service(get_episode_actions)
        .service(upload_episode_actions)
}


