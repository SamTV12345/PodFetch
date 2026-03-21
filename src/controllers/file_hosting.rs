use crate::app_state::AppState;
use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::utils::podcast_key_checker::check_permissions_for_files;
use axum::middleware::from_fn_with_state;
use axum::routing::get;
use tower_http::services::ServeDir;
use utoipa_axum::router::OpenApiRouter;

pub fn podcast_serving(state: AppState) -> OpenApiRouter {
    OpenApiRouter::new().nest(
        "/podcasts",
        OpenApiRouter::new()
            .route("/trololol", get(|| async { "trololol" }))
            .fallback_service(ServeDir::new(&ENVIRONMENT_SERVICE.default_podfetch_folder))
            .route_layer(from_fn_with_state(state, check_permissions_for_files)),
    )
}
