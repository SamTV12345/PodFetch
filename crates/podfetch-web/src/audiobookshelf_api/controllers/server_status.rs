use crate::app_state::AppState;
use axum::Json;
use serde::Serialize;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[derive(Serialize, utoipa::ToSchema)]
pub struct PingResponse {
    pub success: bool,
}

#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StatusResponse {
    pub is_init: bool,
    pub language: String,
    pub auth_methods: Vec<String>,
    pub auth_form_data: Option<serde_json::Value>,
    pub server_version: String,
}

#[utoipa::path(
    get,
    path = "/ping",
    responses((status = 200, description = "Ping", body = PingResponse)),
    tag = "audiobookshelf"
)]
pub async fn ping() -> Json<PingResponse> {
    Json(PingResponse { success: true })
}

#[utoipa::path(
    get,
    path = "/status",
    responses((status = 200, description = "Server status", body = StatusResponse)),
    tag = "audiobookshelf"
)]
pub async fn status() -> Json<StatusResponse> {
    Json(StatusResponse {
        is_init: true,
        language: "en-us".to_string(),
        auth_methods: vec!["local".to_string()],
        auth_form_data: None,
        server_version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

pub fn get_status_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(ping))
        .routes(routes!(status))
}
