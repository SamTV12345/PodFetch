use std::convert::Infallible;
use axum::middleware::from_fn;
use axum::Router;
use axum::routing::post;
use axum::{http::{StatusCode}};
use crate::adapters::api::controllers::device_controller::{get_devices_of_user, post_device};
use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::controllers::api_doc::ApiDoc;
use crate::controllers::file_hosting::get_podcast_serving;
use crate::controllers::manifest_controller::get_manifest;
use crate::controllers::podcast_controller::proxy_podcast;
use crate::controllers::websocket_controller::{
    get_rss_feed, get_rss_feed_for_podcast, start_connection,
};
use crate::gpodder::auth::authentication::login;
use crate::gpodder::parametrization::{get_client_parametrization, get_client_parametrization_router};
use crate::gpodder::subscription::subscriptions::{get_subscriptions, get_subscriptions_all, upload_subscription_changes};
use crate::{get_api_config, get_ui_config};
use actix_web::body::{BoxBody, EitherBody};
use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::web::redirect;
use actix_web::{web, Error, Scope};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn global_routes() -> Router {
    let base_path = ENVIRONMENT_SERVICE
        .sub_directory
        .clone()
        .unwrap_or("/".to_string());
    let openapi = ApiDoc::openapi();
    let service = get_api_config();

    Router::new()
        .nest(&base_path, Router::new()
            .merge(get_client_parametrization_router()))
    web::scope(&base_path)
        .service(get_client_parametrization)
        .service(proxy_podcast)
        .service(get_ui_config())
        .service(get_podcast_serving())
        .service(redirect("/swagger-ui", "/swagger-ui/"))
        .service(SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-doc/openapi.json", openapi))
        .service(redirect("/", "./ui/"))
        .service(service)
        .service(start_connection)
        .service(get_rss_feed)
        .service(get_manifest)
        .service(get_rss_feed_for_podcast)
}

pub fn get_gpodder_api() -> Router {
    if ENVIRONMENT_SERVICE.gpodder_integration_enabled {
        Router::new()
            .route("/auth/{username}/login.json", post(login))
            .service(login)
            .service(get_authenticated_gpodder())
    } else {
        Router::new()
            .layer(from_fn(|_| async {
                log::error!("Gpodder integration is disabled!!");
                StatusCode::NOT_FOUND}))
    }
}

fn get_authenticated_gpodder() -> impl Into<Router> {
    Router::new()
        .layer(crate::gpodder::session_middleware::CookieFilter::new())
        .service(post_device)
        .service(get_devices_of_user)
        .service(get_subscriptions)
        .service(get_subscriptions_all)
        .service(upload_subscription_changes)
        .service(crate::gpodder::episodes::gpodder_episodes::get_episode_actions)
        .service(crate::gpodder::episodes::gpodder_episodes::upload_episode_actions)
}
