use crate::db::DB;
use crate::models::settings::Setting;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use actix_web::web::Data;
use actix_web::{get, put};
use actix_web::{web, HttpResponse, Responder};
use std::sync::Mutex;

#[get("/settings")]
pub async fn get_settings(db: Data<Mutex<DB>>) -> impl Responder {
    let mut db = db.lock().expect("Error acquiring db lock");

    let settings = db.get_settings();
    match settings {
        Some(settings) => HttpResponse::Ok().json(settings),
        None => HttpResponse::NotFound().finish(),
    }
}

#[put("/settings")]
pub async fn update_settings(db: Data<Mutex<DB>>, settings: web::Json<Setting>) -> impl Responder {
    let mut db = db.lock().expect("Error acquiring db lock");

    let settings = db.update_settings(settings.into_inner());
    HttpResponse::Ok().json(settings)
}

#[put("/settings/runcleanup")]
pub async fn run_cleanup(
    pdservice: Data<Mutex<PodcastEpisodeService>>,
    db: Data<Mutex<DB>>,
) -> impl Responder {
    let settings = db.lock().expect("Error acquiring db lock").get_settings();
    match settings {
        Some(settings) => {
            pdservice
                .lock()
                .expect("Error acquiring db lock")
                .cleanup_old_episodes(settings.auto_cleanup_days);
            HttpResponse::Ok().finish()
        }
        None => {
            log::error!("Error getting settings");
            HttpResponse::InternalServerError().finish()
        }
    }
}
