use std::convert::Infallible;
use axum::middleware::from_fn;
use axum::Router;
use axum::routing::post;
use axum::{http::{StatusCode}};
use tower::Layer;
use crate::adapters::api::controllers::device_controller::{get_devices_of_user, post_device};
use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::controllers::manifest_controller::get_manifest;
use crate::controllers::podcast_controller::proxy_podcast;
use crate::controllers::websocket_controller::{
    get_rss_feed, get_rss_feed_for_podcast,
};
use crate::gpodder::auth::authentication::login;
use crate::gpodder::parametrization::{get_client_parametrization, get_client_parametrization_router};
use crate::gpodder::subscription::subscriptions::{get_subscriptions, get_subscriptions_all, upload_subscription_changes};
use crate::{get_api_config, get_ui_config};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use crate::gpodder::session_middleware::CookieFilter;

pub fn global_routes() -> Router {
    let base_path = ENVIRONMENT_SERVICE
        .sub_directory
        .clone()
        .unwrap_or("/".to_string());
    let service = get_api_config();

    Router::new()
        .nest(&base_path, Router::new()
            .merge(get_client_parametrization_router()))
        .merge(get_gpodder_api())
}

pub fn get_gpodder_api() -> Router {
    if ENVIRONMENT_SERVICE.gpodder_integration_enabled {
        Router::new()
            .route("/auth/{username}/login.json", post(login))
            .merge(get_authenticated_gpodder())
    } else {
        Router::new()
            .layer(from_fn(|_, _| async {
                log::error!("Gpodder integration is disabled!!");
                StatusCode::NOT_FOUND
            }))
    }
}

fn get_authenticated_gpodder() -> impl Into<Router> {

    Router::new()
        .layer(CookieFilter::)
}
