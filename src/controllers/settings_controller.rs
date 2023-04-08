use crate::models::settings::Setting;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use actix_web::web::Data;
use actix_web::{get, put};
use actix_web::{web, HttpResponse, Responder};
use std::sync::Mutex;
use crate::DbPool;
use crate::mutex::LockResultExt;
use crate::service::settings_service::SettingsService;

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets the current settings")),
tag="podcast_episodes"
)]
#[get("/settings")]
pub async fn get_settings(db: Data<Mutex<SettingsService>>) -> impl Responder {
    let mut db = db.lock().ignore_poison();

    let settings = db.get_settings();
    match settings {
        Some(settings) => HttpResponse::Ok().json(settings),
        None => HttpResponse::NotFound().finish(),
    }
}

#[utoipa::path(
context_path="/api/v1",
request_body=Setting,
responses(
(status = 200, description = "Updates the current settings")),
tag="settings"
)]
#[put("/settings")]
pub async fn update_settings(db: Data<Mutex<SettingsService>>, settings: web::Json<Setting>) -> impl Responder {
    let mut db = db.lock().ignore_poison();

    let settings = db.update_settings(settings.into_inner());
    HttpResponse::Ok().json(settings)
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Runs a cleanup of old episodes")),
tag="settings"
)]
#[put("/settings/runcleanup")]
pub async fn run_cleanup(
    pdservice: Data<Mutex<PodcastEpisodeService>>,
    db: Data<Mutex<SettingsService>>,
    conn: Data<DbPool>
) -> impl Responder {
    let settings = db.lock().ignore_poison().get_settings();
    match settings {
        Some(settings) => {
            pdservice
                .lock()
                .ignore_poison()
                .cleanup_old_episodes(settings.auto_cleanup_days,&mut conn.get().unwrap());
            HttpResponse::Ok().finish()
        }
        None => {
            log::error!("Error getting settings");
            HttpResponse::InternalServerError().finish()
        }
    }
}
