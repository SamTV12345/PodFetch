use crate::app_state::AppState;
use crate::gpodder_api::auth::authentication::login;
use crate::gpodder_api::device::device_controller::get_device_router;
use crate::gpodder_api::episodes::gpodder_episodes::get_gpodder_episodes_router;
use crate::gpodder_api::session_middleware::handle_cookie_session;
use crate::url_rewriting::resolve_server_url_from_headers;
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
use crate::gpodder_api::subscription::subscriptions::get_subscription_router;
use axum::http::HeaderMap;
use axum::Json;
use axum::middleware::from_fn_with_state;
use axum::routing::get;
use crate::gpodder::{ClientParametrization, build_client_parametrization};
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
            .nest(
                "/api/2",
                OpenApiRouter::new()
                    .merge(get_subscription_router().with_state(state.clone()))
                    .merge(get_device_router().with_state(state.clone()))
                    .merge(get_gpodder_episodes_router())
                    .layer(from_fn_with_state(state.clone(), handle_cookie_session)),
            );
    }
    router
}
