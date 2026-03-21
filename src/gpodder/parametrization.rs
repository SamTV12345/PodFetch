use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use axum::Json;
use axum::routing::get;
use podfetch_web::gpodder::{ClientParametrization, build_client_parametrization};
use utoipa_axum::router::OpenApiRouter;

pub async fn get_client_parametrization() -> Json<ClientParametrization> {
    let answer = build_client_parametrization(&ENVIRONMENT_SERVICE.server_url);

    Json(answer)
}

pub fn get_client_parametrization_router() -> OpenApiRouter {
    OpenApiRouter::new().route("/clientconfig.json", get(get_client_parametrization))
}
