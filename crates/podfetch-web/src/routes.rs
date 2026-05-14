use crate::app_state::AppState;
use crate::audiobookshelf_api::auth_middleware::audiobookshelf_bearer_auth;
use crate::audiobookshelf_api::controllers::auth::{
    get_auth_router as get_audiobookshelf_auth_router,
    get_authorize_router as get_audiobookshelf_authorize_router,
};
use crate::audiobookshelf_api::controllers::hls::get_hls_router as get_audiobookshelf_hls_router;
use crate::audiobookshelf_api::controllers::items::get_items_router as get_audiobookshelf_items_router;
use crate::audiobookshelf_api::controllers::libraries::get_libraries_router as get_audiobookshelf_libraries_router;
use crate::audiobookshelf_api::controllers::me::get_me_router as get_audiobookshelf_me_router;
use crate::audiobookshelf_api::controllers::playlists::get_playlists_router as get_audiobookshelf_playlists_router;
use crate::audiobookshelf_api::controllers::podcasts::get_podcasts_router as get_audiobookshelf_podcasts_router;
use crate::audiobookshelf_api::controllers::public_session::get_public_session_router as get_audiobookshelf_public_session_router;
use crate::audiobookshelf_api::controllers::search::get_search_router as get_audiobookshelf_search_router;
use crate::audiobookshelf_api::controllers::scan::get_scan_router as get_audiobookshelf_scan_router;
use crate::audiobookshelf_api::controllers::server_status::get_status_router as get_audiobookshelf_status_router;
use crate::audiobookshelf_api::controllers::sessions::get_sessions_router as get_audiobookshelf_sessions_router;
use crate::audiobookshelf_api::controllers::uploads::get_upload_router as get_audiobookshelf_upload_router;
use crate::gpodder::{ClientParametrization, build_client_parametrization};
use crate::gpodder_api::auth::authentication::get_auth_router;
use crate::gpodder_api::device::device_controller::get_device_router;
use crate::gpodder_api::episodes::gpodder_episodes::get_gpodder_episodes_router;
use crate::gpodder_api::session_middleware::handle_cookie_session;
use crate::gpodder_api::settings::settings_controller::get_settings_router;
use crate::gpodder_api::subscription::subscriptions::{
    get_simple_subscription_router, get_subscription_router, put_device_subscriptions,
    put_simple_subscriptions,
};
use crate::url_rewriting::resolve_server_url_from_headers;
use axum::Json;
use axum::http::HeaderMap;
use axum::middleware::from_fn_with_state;
use axum::routing::{get, put};
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
use utoipa_axum::router::OpenApiRouter;

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
        let gpodder_session = from_fn_with_state(state.clone(), handle_cookie_session);

        router = router
            // Auth endpoints (login + logout) — no session middleware needed
            .merge(get_auth_router().with_state(state.clone()))
            // Simple API (for Kodi and other clients expecting gpodder.net-style endpoints)
            .nest(
                "/subscriptions",
                OpenApiRouter::new()
                    .merge(get_simple_subscription_router().with_state(state.clone()))
                    .route(
                        "/{username}/{deviceid}",
                        put(put_simple_subscriptions).with_state(state.clone()),
                    )
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
                    .route(
                        "/subscriptions/{username}/{deviceid}",
                        put(put_device_subscriptions).with_state(state.clone()),
                    )
                    .layer(gpodder_session),
            );
    }

    if ENVIRONMENT_SERVICE.audiobookshelf_integration_enabled {
        // Audiobookshelf-compatible surface. Endpoints sit at root paths so the
        // official mobile apps (which hardcode `/login`, `/api/...`, `/public/...`,
        // `/hls/...`, `/socket.io/`) work without a path prefix.
        let abs_bearer = from_fn_with_state(state.clone(), audiobookshelf_bearer_auth);

        let abs_unauth = OpenApiRouter::new()
            .merge(get_audiobookshelf_status_router().with_state(state.clone()))
            .merge(get_audiobookshelf_auth_router().with_state(state.clone()));

        let abs_auth = OpenApiRouter::new()
            .merge(get_audiobookshelf_libraries_router().with_state(state.clone()))
            .merge(get_audiobookshelf_scan_router().with_state(state.clone()))
            .merge(get_audiobookshelf_items_router().with_state(state.clone()))
            .merge(get_audiobookshelf_me_router().with_state(state.clone()))
            .merge(get_audiobookshelf_sessions_router().with_state(state.clone()))
            .merge(get_audiobookshelf_upload_router().with_state(state.clone()))
            .merge(get_audiobookshelf_public_session_router().with_state(state.clone()))
            .merge(get_audiobookshelf_hls_router().with_state(state.clone()))
            .merge(get_audiobookshelf_authorize_router().with_state(state.clone()))
            .merge(get_audiobookshelf_search_router().with_state(state.clone()))
            .merge(get_audiobookshelf_podcasts_router().with_state(state.clone()))
            .merge(get_audiobookshelf_playlists_router().with_state(state.clone()))
            .layer(abs_bearer);

        router = router.merge(abs_unauth).merge(abs_auth);
    }
    router
}
