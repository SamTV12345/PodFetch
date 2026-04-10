use crate::app_state::AppState;
use crate::gpodder::{ClientParametrization, build_client_parametrization};
use crate::gpodder_api::auth::authentication::{get_auth_router, login};
use crate::gpodder_api::device::device_controller::get_device_router;
use crate::gpodder_api::episodes::gpodder_episodes::get_gpodder_episodes_router;
use crate::gpodder_api::session_middleware::handle_cookie_session;
use crate::gpodder_api::settings::settings_controller::get_settings_router;
use crate::gpodder_api::subscription::subscriptions::{
    get_simple_subscription_router, get_subscription_router,
};
use crate::url_rewriting::resolve_server_url_from_headers;
use axum::Json;
use axum::http::HeaderMap;
use axum::middleware::from_fn_with_state;
use axum::routing::get;
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

pub async fn get_client_parametrization(headers: HeaderMap) -> Json<ClientParametrization> {
    let server_url = resolve_server_url_from_headers(&headers);
    let answer = build_client_parametrization(&server_url);
    Json(answer)
}

pub fn get_client_parametrization_router() -> OpenApiRouter {
    OpenApiRouter::new().route("/clientconfig.json", get(get_client_parametrization))
}

pub fn global_routes(state: AppState, api_config: OpenApiRouter) -> OpenApiRouter {
    let base_path = ENVIRONMENT_SERVICE
        .sub_directory
        .clone()
        .unwrap_or_default();
    let service = api_config;

    let inner = OpenApiRouter::new()
        .merge(get_client_parametrization_router())
        .merge(service);

    let mut router = if base_path.is_empty() || base_path == "/" {
        OpenApiRouter::new().merge(inner)
    } else {
        OpenApiRouter::new().nest(&base_path, inner)
    };

    if ENVIRONMENT_SERVICE.gpodder_integration_enabled {
        use crate::gpodder_api::auth::authentication::__path_login;
        router = router
            .merge(
                OpenApiRouter::new()
                    .routes(routes!(login))
                    .with_state(state.clone()),
            )
            // Simple API (for Kodi and other clients expecting gpodder.net-style endpoints)
            .nest(
                "/subscriptions",
                OpenApiRouter::new()
                    .merge(get_simple_subscription_router().with_state(state.clone()))
                    .layer(from_fn_with_state(state.clone(), handle_cookie_session)),
            )
            // Advanced API (v2)
            .nest(
                "/api/2",
                OpenApiRouter::new()
                    .merge(get_subscription_router().with_state(state.clone()))
                    .merge(get_device_router().with_state(state.clone()))
                    .merge(get_gpodder_episodes_router().with_state(state.clone()))
                    .merge(get_settings_router().with_state(state.clone()))
                    .merge(get_auth_router().with_state(state.clone()))
                    .layer(from_fn_with_state(state.clone(), handle_cookie_session)),
            );
    }
    router
}
