use crate::gpodder::auth::authentication::login;
use crate::gpodder::device::device_controller::{get_devices_of_user, post_device};
use crate::gpodder::episodes::gpodder_episodes::{get_episode_actions, upload_episode_actions};
use crate::gpodder::session_middleware::CookieFilter;
use crate::gpodder::subscription::subscriptions::{get_subscriptions, upload_subscription_changes};

use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use actix_web::body::{BoxBody, EitherBody};
use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::{web, Error, Scope};

pub fn get_gpodder_api() -> Scope {
    if ENVIRONMENT_SERVICE
        .get()
        .unwrap()
        .gpodder_integration_enabled
    {
        web::scope("/api/2")
            .service(login)
            .service(get_authenticated_api())
    } else {
        web::scope("/api/2")
    }
}

pub fn get_authenticated_api() -> Scope<
    impl ServiceFactory<
        ServiceRequest,
        Config = (),
        Response = ServiceResponse<EitherBody<BoxBody>>,
        Error = Error,
        InitError = (),
    >,
> {
    web::scope("")
        .wrap(CookieFilter::new())
        .service(post_device)
        .service(get_devices_of_user)
        .service(get_subscriptions)
        .service(upload_subscription_changes)
        .service(get_episode_actions)
        .service(upload_episode_actions)
}
