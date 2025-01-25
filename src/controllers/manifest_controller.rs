use axum::Json;
use axum::routing::get;
use utoipa_axum::router::OpenApiRouter;
use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::utils::error::CustomError;

#[derive(Serialize)]
pub struct Icon {
    pub src: String,
    pub sizes: String,
    pub r#type: String,
}

#[derive(Serialize)]
pub struct Manifest {
    pub name: String,
    pub short_name: String,
    pub start_url: String,
    pub icons: Vec<Icon>,
    pub theme_color: String,
    pub background_color: String,
    pub display: String,
    pub orientation: String,
}


pub async fn get_manifest() -> Result<Json<Manifest>, CustomError> {
    let mut icons = Vec::new();
    let icon = Icon {
        src: ENVIRONMENT_SERVICE.server_url.to_string() + "ui/logo.png",
        sizes: "512x512".to_string(),
        r#type: "image/png".to_string(),
    };
    icons.push(icon);

    let manifest = Manifest {
        name: "PodFetch".to_string(),
        short_name: "PodFetch".to_string(),
        start_url: ENVIRONMENT_SERVICE.server_url.to_string(),
        icons,
        orientation: "landscape".to_string(),
        theme_color: "#ffffff".to_string(),
        display: "fullscreen".to_string(),
        background_color: "#ffffff".to_string(),
    };
    Ok(Json(manifest))
}

pub fn get_manifest_router() -> OpenApiRouter {
    OpenApiRouter::new().route("/manifest.json", get(get_manifest))
}