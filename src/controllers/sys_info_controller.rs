use crate::constants::constants::ERROR_LOGIN_MESSAGE;
use crate::service::environment_service::EnvironmentService;
use actix_web::web::Data;
use actix_web::{get, post};
use actix_web::{web, HttpResponse, Responder};
use fs_extra::dir::get_size;
use std::sync::{Mutex};
use sysinfo::{System, SystemExt};
use crate::mutex::LockResultExt;

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets the system information",
body = SysExtraInfo)),
tag="sys"
)]
#[get("/sys/info")]
pub async fn get_sys_info() -> impl Responder {
    let mut sys = System::new_all();
    sys.refresh_all();

    let podcast_byte_size = get_size("podcasts").unwrap();
    HttpResponse::Ok().json(SysExtraInfo {
        system: sys,
        podcast_directory: podcast_byte_size,
    })
}

#[derive(Debug, Serialize)]
pub struct SysExtraInfo {
    pub system: System,
    pub podcast_directory: u64,
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets the environment configuration",
body=SysExtraInfo)),
tag="sys"
)]
#[get("/sys/config")]
pub async fn get_public_config() -> impl Responder {
    let mut env = EnvironmentService::new();
    let config = env.get_config();
    HttpResponse::Ok().json(config)
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Performs a login if basic auth is enabled",
body=String)),
tag="sys"
)]
#[post("/login")]
pub async fn login(
    auth: web::Json<LoginRequest>,
    env: Data<Mutex<EnvironmentService>>,
) -> impl Responder {
    let env_service = env.lock().ignore_poison();

    if auth.0.username == env_service.username && auth.0.password == env_service.password {
        return HttpResponse::Ok().json("Login successful");
    }

    HttpResponse::Unauthorized().json(ERROR_LOGIN_MESSAGE)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}
