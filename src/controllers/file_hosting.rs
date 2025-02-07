use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::utils::podcast_key_checker::check_permissions_for_files;
use axum::middleware::from_fn;
use axum::routing::get;
use tower_http::services::ServeDir;
use utoipa_axum::router::OpenApiRouter;

pub fn podcast_serving() -> OpenApiRouter {
    OpenApiRouter::new().nest(
        "/podcasts",
        OpenApiRouter::new()
            .route("/trololol", get(|| async { "trololol" }))
            .fallback_service(ServeDir::new(&ENVIRONMENT_SERVICE.default_podfetch_folder))
            .route_layer(from_fn(check_permissions_for_files)),
    )
}
