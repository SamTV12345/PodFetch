use axum::middleware::from_fn;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::gpodder::auth::authentication::login;
use crate::gpodder::parametrization::{get_client_parametrization_router};
use crate::{get_api_config};
use crate::gpodder::device::device_controller::get_device_router;
use crate::gpodder::episodes::gpodder_episodes::get_gpodder_episodes_router;
use crate::gpodder::session_middleware::handle_cookie_session;
use crate::gpodder::subscription::subscriptions::get_subscription_router;

pub fn global_routes() -> OpenApiRouter {
    let base_path = ENVIRONMENT_SERVICE
        .sub_directory
        .clone()
        .unwrap_or("/".to_string());
    let service = get_api_config();

    let mut router = match base_path.is_empty() {
        true=>{
            OpenApiRouter::new()
                    .merge(get_client_parametrization_router())
                    .merge(service)
        }
        false=>{
            OpenApiRouter::new()
                .nest(&base_path, OpenApiRouter::new()
                    .merge(get_client_parametrization_router())
                    .merge(service))
        }
    };

    if ENVIRONMENT_SERVICE.gpodder_integration_enabled {
        use crate::gpodder::auth::authentication::__path_login;
        router = router.nest("/api/2",OpenApiRouter::new()
            .routes(routes!(login))
            .merge(get_subscription_router())
            .merge(get_device_router())
            .merge(get_gpodder_episodes_router())
            .layer(from_fn(handle_cookie_session))
        );
    }
    router
}