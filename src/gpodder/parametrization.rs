use actix_web::get;

use actix_web::{HttpResponse, Responder};

use crate::constants::inner_constants::ENVIRONMENT_SERVICE;

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

#[get("/clientconfig.json")]
pub async fn get_client_parametrization() -> impl Responder {
    let answer = ClientParametrization {
        mygpo_feedservice: BaseURL {
            base_url: ENVIRONMENT_SERVICE.server_url.clone(),
        },
        mygpo: BaseURL {
            base_url: ENVIRONMENT_SERVICE.server_url.to_string() + "rss",
        },
        update_timeout: 604800,
    };

    HttpResponse::Ok().json(answer)
}
