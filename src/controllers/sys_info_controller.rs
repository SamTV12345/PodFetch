use crate::models::user::User;

use actix_web::{get, post};
use actix_web::{web, HttpResponse, Responder};
use fs_extra::dir::get_size;
use sha256::digest;

use sysinfo::{Disk, Disks, System};
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
pub async fn get_sys_info() -> Result<HttpResponse, CustomError> {
    let mut sys = System::new();
    let disks = Disks::new_with_refreshed_list();

    let sim_disks = disks
        .iter()
        .map(|disk| disk.into())
        .collect::<Vec<SimplifiedDisk>>();

    sys.refresh_all();
    sys.refresh_cpu_all();

    const PATH: &str = "podcasts";
    let podcast_byte_size =
        get_size(PATH).map_err(|e| map_io_extra_error(e, Some(PATH.to_string())))?;
    Ok(HttpResponse::Ok().json(SysExtraInfo {
        system: sys.into(),
        disks: sim_disks,
        podcast_directory: podcast_byte_size,
    }))
}
use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::utils::error::{map_io_extra_error, CustomError};
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct SysExtraInfo {
    pub system: SystemDto,
    pub disks: Vec<SimplifiedDisk>,
    pub podcast_directory: u64,
}

impl From<System> for SystemDto {
    fn from(sys: System) -> Self {
        SystemDto {
            mem_total: sys.total_memory(),
            mem_available: sys.available_memory(),
            swap_total: sys.total_swap(),
            swap_used: sys.used_swap(),
            cpus: CpusWrapperDto {
                global: sys.global_cpu_usage(),
                cpus: sys
                    .cpus()
                    .iter()
                    .map(|cpu| Cpu {
                        name: cpu.name().to_string(),
                        vendor_id: cpu.vendor_id().to_string(),
                        usage: CpuUsageDto {
                            percent: cpu.cpu_usage(),
                        },
                        brand: cpu.brand().to_string(),
                        frequency: cpu.frequency(),
                    })
                    .collect(),
            },
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SystemDto {
    mem_total: u64,
    mem_available: u64,
    swap_total: u64,
    swap_used: u64,
    cpus: CpusWrapperDto,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CpusWrapperDto {
    global: f32,
    cpus: Vec<Cpu>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Cpu {
    name: String,
    vendor_id: String,
    usage: CpuUsageDto,
    brand: String,
    frequency: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CpuUsageDto {
    percent: f32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SimplifiedDisk {
    pub name: String,
    pub total_space: u64,
    pub available_space: u64,
}

impl From<&Disk> for SimplifiedDisk {
    fn from(disk: &Disk) -> Self {
        SimplifiedDisk {
            name: disk.name().to_str().unwrap_or("").to_string(),
            total_space: disk.total_space(),
            available_space: disk.available_space(),
        }
    }
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
    let config = ENVIRONMENT_SERVICE.get().unwrap().get_config();
    HttpResponse::Ok().json(config)
}

#[utoipa::path(
context_path="/api/v1",
request_body=LoginRequest,
responses(
(status = 200, description = "Performs a login if basic auth is enabled",
body=String)),
tag="sys"
)]
#[post("/login")]
pub async fn login(auth: web::Json<LoginRequest>) -> Result<HttpResponse, CustomError> {
    use crate::ENVIRONMENT_SERVICE;
    let env_service = ENVIRONMENT_SERVICE.get().unwrap();

    let digested_password = digest(auth.0.password);
    if let Some(admin_username) = &env_service.username {
        if admin_username == &auth.0.username {
            if let Some(admin_password) = &env_service.password {
                if admin_password == &digested_password {
                    return Ok(HttpResponse::Ok().json("Login successful"));
                }
            }
        }
    }
    let db_user = User::find_by_username(&auth.0.username)?;

    if db_user.password.is_none() {
        log::warn!("Login failed for user {}", auth.0.username);
        return Err(CustomError::Forbidden);
    }

    if db_user.password.unwrap() == digested_password {
        return Ok(HttpResponse::Ok().json("Login successful"));
    }
    log::warn!("Login failed for user {}", auth.0.username);
    Err(CustomError::Forbidden)
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
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
    pub time: &'static str,
    pub os: &'static str,
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets the info of the server")),
tag="info"
)]
#[get("/info")]
pub async fn get_info() -> impl Responder {
    let version = VersionInfo {
        commit: env!("GIT_EXACT_TAG"),
        version: env!("VW_VERSION"),
        r#ref: env!("GIT_BRANCH"),
        ci: built_info::CI_PLATFORM.unwrap_or("No CI platform"),
        time: built_info::BUILT_TIME_UTC,
        os: built_info::CFG_OS,
    };
    HttpResponse::Ok().json(version)
}
