use axum::middleware::from_fn;
use axum::Router;
use axum::routing::get;
use tower_http::services::ServeDir;
use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::utils::podcast_key_checker::check_permissions_for_files;

pub fn podcast_serving() -> impl Into<Router> {

    Router::new()
        .nest("/podcasts", Router::new()
            .route("/trololol", get(||{
                async {
                    "trololol"
                }
            }))
            .fallback_service(ServeDir::new(ENVIRONMENT_SERVICE.default_podfetch_folder.to_string()))
        .route_layer(from_fn(check_permissions_for_files)))

}
