use crate::models::user::User;
use axum::{Extension, Json};

use fs_extra::dir::get_size;
use reqwest::StatusCode;
use sha256::digest;

use sysinfo::{Disk, Disks, System};
pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[utoipa::path(
get,
path="/sys/info",
responses(
(status = 200, description = "Gets the system information",
body = SysExtraInfo)),
tag="sys"
)]
pub async fn get_sys_info(
    Extension(requester): Extension<User>,
) -> Result<Json<SysExtraInfo>, CustomError> {
    if !requester.is_admin() {
        return Err(CustomErrorInner::Forbidden(Info).into());
    }

    let mut sys = System::new();
    let disks = Disks::new_with_refreshed_list();

    let sim_disks = disks
        .iter()
        .map(|disk| disk.into())
        .collect::<Vec<SimplifiedDisk>>();

    sys.refresh_all();
    sys.refresh_cpu_all();

    let podcast_byte_size =
        get_size(&ENVIRONMENT_SERVICE.default_podfetch_folder).map_err(|e| {
            map_io_extra_error(
                e,
                Some(
                    ENVIRONMENT_SERVICE
                        .default_podfetch_folder
                        .to_string()
                        .to_string(),
                ),
                ErrorSeverity::Critical,
            )
        })?;
    Ok(Json(SysExtraInfo {
        system: sys.into(),
        disks: sim_disks,
        podcast_directory: podcast_byte_size,
    }))
}
use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
use crate::models::settings::ConfigModel;
use crate::utils::error::ErrorSeverity::Info;
use crate::utils::error::ErrorType::CustomErrorType;
use crate::utils::error::{
    ApiError, CustomError, CustomErrorInner, ErrorSeverity, ErrorType, map_io_extra_error,
};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

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
get,
path="/sys/config",
responses(
(status = 200, description = "Gets the environment configuration",
body=ConfigModel)),
tag="sys"
)]

pub async fn get_public_config() -> Json<ConfigModel> {
    let config = ENVIRONMENT_SERVICE.get_config();
    Json(config)
}

#[utoipa::path(
post,
path="/login",
request_body=LoginRequest,
responses(
(status = 200, description = "Performs a login if basic auth is enabled",
body=String)),
tag="sys"
)]
pub async fn login(auth: Json<LoginRequest>) -> Result<StatusCode, ErrorType> {
    use crate::ENVIRONMENT_SERVICE;

    let digested_password = digest(auth.0.password);
    if let Some(admin_username) = &ENVIRONMENT_SERVICE.username
        && admin_username == &auth.0.username
        && let Some(admin_password) = &ENVIRONMENT_SERVICE.password
        && admin_password == &digested_password
    {
        return Ok(StatusCode::OK);
    }
    let db_user = match User::find_by_username(&auth.0.username) {
        Ok(user) => user,
        Err(err) => {
            if matches!(err.inner, CustomErrorInner::NotFound(_)) {
                log::warn!("Login failed for user {}", auth.0.username);
                return Err(ApiError::wrong_user_or_password().into());
            }
            return Err(CustomErrorType(err));
        }
    };

    if db_user.password.is_none() {
        log::warn!("Login failed for user {}", auth.0.username);
        return Err(CustomErrorInner::Forbidden(Info).into());
    }

    if db_user.password.unwrap() == digested_password {
        return Ok(StatusCode::OK);
    }
    log::warn!("Login failed for user {}", auth.0.username);
    Err(CustomErrorInner::Forbidden(Info).into())
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct VersionInfo {
    pub version: &'static str,
    pub r#ref: &'static str,
    pub commit: &'static str,
    pub ci: &'static str,
    pub time: &'static str,
    pub os: &'static str,
}

#[utoipa::path(
get,
path="/info",
responses(
(status = 200, description = "Gets the info of the server", body=VersionInfo)),
tag="info"
)]
pub async fn get_info() -> Json<VersionInfo> {
    let version = VersionInfo {
        commit: env!("GIT_EXACT_TAG"),
        version: env!("VW_VERSION"),
        r#ref: env!("GIT_BRANCH"),
        ci: built_info::CI_PLATFORM.unwrap_or("No CI platform"),
        time: built_info::BUILT_TIME_UTC,
        os: built_info::CFG_OS,
    };
    Json(version)
}

pub fn get_sys_info_router() -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(get_sys_info))
        .routes(routes!(get_info))
}
