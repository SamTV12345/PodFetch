use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use axum::routing::get;
use axum::Json;
use utoipa_axum::router::OpenApiRouter;

#[derive(Serialize, Deserialize)]
pub struct ClientParametrization {
    mygpo: BaseURL,
    #[serde(rename = "mygpo-feedservice")]
    mygpo_feedservice: BaseURL,
    update_timeout: i32,
}

#[derive(Serialize, Deserialize)]
pub struct BaseURL {
    #[serde(rename = "baseurl")]
    base_url: String,
}

pub async fn get_client_parametrization() -> Json<ClientParametrization> {
    let answer = ClientParametrization {
        mygpo_feedservice: BaseURL {
            base_url: ENVIRONMENT_SERVICE.server_url.clone(),
        },
        mygpo: BaseURL {
            base_url: ENVIRONMENT_SERVICE.server_url.to_string() + "rss",
        },
        update_timeout: 604800,
    };

    Json(answer)
}

pub fn get_client_parametrization_router() -> OpenApiRouter {
    OpenApiRouter::new().route("/clientconfig.json", get(get_client_parametrization))
}
