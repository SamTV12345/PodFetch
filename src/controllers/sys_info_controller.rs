use crate::constants::constants::ERROR_LOGIN_MESSAGE;
use crate::service::environment_service::EnvironmentService;
use actix_web::web::Data;
use actix_web::{get, post};
use actix_web::{web, HttpResponse, Responder};
use fs_extra::dir::get_size;
use std::sync::{Mutex};
use sysinfo::{System, SystemExt};
use crate::models::user::User;
use crate::mutex::LockResultExt;
use sha256::{digest};
use crate::DbPool;
pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

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
    db: Data<DbPool>
) -> impl Responder {
    let env_service = env.lock().ignore_poison();

    if auth.0.username == env_service.username && auth.0.password == env_service.password {
        return HttpResponse::Ok().json("Login successful");
    }
    let db_user = User::find_by_username(&auth.0.username, &mut *db.get().unwrap());

    return match db_user {
        Some(user) => {
            if user.password.unwrap() == digest(auth.0.password) {
                return HttpResponse::Ok().json("Login successful");
            }
            HttpResponse::Unauthorized().json(ERROR_LOGIN_MESSAGE)
        }
        None => {
            HttpResponse::Unauthorized().json(ERROR_LOGIN_MESSAGE)
        }
    };
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VersionInfo {
    pub version: &'static str,
    pub r#ref: &'static str,
    pub commit: &'static str,
    pub ci: &'static str,
    pub time:&'static str,
    pub os: &'static str,
}

#[get("/info")]
pub async fn get_info() -> impl Responder {
    let version = VersionInfo{
        commit: built_info::GIT_COMMIT_HASH.unwrap_or("No commit hash"),
        version: built_info::GIT_VERSION.unwrap_or("No git version"),
        r#ref: built_info::GIT_HEAD_REF.unwrap_or("No github ref"),
        ci: built_info::CI_PLATFORM.unwrap_or("No CI platform"),
        time: built_info::BUILT_TIME_UTC,
        os: built_info::CFG_OS,
    };
    HttpResponse::Ok().json(version)
}