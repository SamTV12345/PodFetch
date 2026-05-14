use crate::app_state::AppState;
use axum::Json;
use serde::Serialize;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[derive(Serialize, utoipa::ToSchema)]
pub struct PingResponse {
    pub success: bool,
}

/// 100 % audiobookshelf-shape per upstream `Server.js:348` /status route.
/// The `app: "audiobookshelf"` field is what the mobile apps probe to
/// confirm they're talking to a compatible server - omitting it makes
/// them refuse the connection.
#[derive(Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StatusResponse {
    pub app: String,
    pub server_version: String,
    pub is_init: bool,
    pub language: String,
    pub auth_methods: Vec<String>,
    pub auth_form_data: Option<serde_json::Value>,
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
        app: "audiobookshelf".to_string(),
        server_version: env!("CARGO_PKG_VERSION").to_string(),
        is_init: true,
        language: "en-us".to_string(),
        auth_methods: vec!["local".to_string()],
        auth_form_data: None,
    })
}

pub fn get_status_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(ping))
        .routes(routes!(status))
}
