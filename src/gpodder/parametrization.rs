use crate::mutex::LockResultExt;
use crate::service::environment_service::EnvironmentService;
use actix_web::get;
use actix_web::web::Data;
use actix_web::{HttpResponse, Responder};
use std::sync::Mutex;

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
pub async fn get_client_parametrization(
    environment_service: Data<Mutex<EnvironmentService>>,
) -> impl Responder {
    let env_service = environment_service.lock().ignore_poison().clone();
    let answer = ClientParametrization {
        mygpo_feedservice: BaseURL {
            base_url: env_service.server_url.clone(),
        },
        mygpo: BaseURL {
            base_url: env_service.server_url + "rss",
        },
        update_timeout: 604800,
    };

    HttpResponse::Ok().json(answer)
}
