use actix_web::{HttpResponse, Responder};
use sysinfo::{System, SystemExt};
use actix_web::get;
use fs_extra::dir::get_size;

#[get("/sys/info")]
pub async fn get_sys_info() -> impl Responder {
    let mut sys = System::new_all();
    sys.refresh_all();

    let podcast_byte_size = get_size("podcasts").unwrap();
    HttpResponse::Ok().json(SysExtraInfo{
        system: sys,
        podcast_directory: podcast_byte_size,
    })
}

#[derive(Debug, Serialize)]
pub struct SysExtraInfo {
    pub system: System,
    pub podcast_directory: u64,
}