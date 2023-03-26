#![feature(slice_concat_trait)]

use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::{env, thread};
use std::sync::{Mutex};
use actix_web::{App, http, HttpResponse, HttpServer, Responder, Scope, web};
use std::time::Duration;
use actix::{Actor};
use actix_cors::Cors;
use actix_files::Files;
use actix_web::web::{Data, redirect};
use clokwerk::{Scheduler, TimeUnits};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod controllers;
pub use controllers::controller_utils::*;
use crate::config::dbconfig::establish_connection;
use crate::controllers::api_doc::ApiDoc;
use crate::controllers::podcast_controller::{download_podcast, favorite_podcast, get_favored_podcasts, query_for_podcast};
use crate::controllers::notification_controller::{dismiss_notifications, get_unread_notifications};
use crate::controllers::podcast_controller::{add_podcast, find_all_podcasts, find_podcast, find_podcast_by_id};
use crate::controllers::podcast_episode_controller::{download_podcast_episodes_of_podcast, find_all_podcast_episodes_of_podcast};
use crate::controllers::sys_info_controller::get_sys_info;
use crate::controllers::watch_time_controller::{get_last_watched, get_watchtime, log_watchtime};
use crate::controllers::websocket_controller::{get_rss_feed, start_connection};
mod db;
mod models;
mod constants;
mod service;
use crate::db::DB;
use crate::models::web_socket_message::Lobby;
use crate::service::environment_service::EnvironmentService;
use crate::service::file_service::FileService;
use crate::service::logging_service::init_logging;
use crate::service::mapping_service::MappingService;
use crate::service::podcast_episode_service::PodcastEpisodeService;
use crate::service::rust_service::PodcastService;

mod config;
pub mod schema;
pub mod utils;


pub fn run_poll(mut podcast_service: PodcastService, mut podcast_episode_service: PodcastEpisodeService){
        let mut db = DB::new().unwrap();
        //check for new episodes
        let podcats_result = db.get_podcasts().unwrap();
        for podcast in podcats_result {
            let podcast_clone = podcast.clone();
            podcast_episode_service.insert_podcast_episodes(podcast);
            podcast_service.schedule_episode_download(podcast_clone, None);
    }
}

async fn index() ->  impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/static/index.html")))
}


pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

#[actix_web::main]
async fn main()-> std::io::Result<()> {

    let lobby = Lobby::default();

    let chat_server = lobby.start();

    let mut connection = establish_connection();
    connection.run_pending_migrations(MIGRATIONS).unwrap();

    EnvironmentService::print_banner();
    init_logging();
    FileService::create_podcast_root_directory_exists();

    //services
    let podcast_episode_service = PodcastEpisodeService::new();
    let podcast_service = PodcastService::new();
    let db = DB::new().unwrap();
    let mapping_service = MappingService::new();
    let file_service = FileService::new();

    thread::spawn(||{
        let mut scheduler = Scheduler::new();
        let env = EnvironmentService::new();
        env.get_environment();
        let polling_interval = env.get_polling_interval();
        scheduler.every(polling_interval.minutes()).run(||{
            let podcast_service = PodcastService::new();
            let podcast_episode_service = PodcastEpisodeService::new();
            log::info!("Polling for new episodes");
           run_poll(podcast_service, podcast_episode_service);
        });
        loop {
            scheduler.run_pending();
            thread::sleep(Duration::from_millis(1000));
        }
    });


    HttpServer::new(move || {
        let openapi = ApiDoc::openapi();

        App::new().service(Files::new
            ("/podcasts", "podcasts").show_files_listing())
            .service(redirect("/swagger-ui", "/swagger-ui/"))
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-doc/openapi.json", openapi),
            )
            .service(redirect("/","/ui/"))
            .wrap(get_cors_config())
            .service(get_api_config())
            .service(get_ui_config())
            .service(start_connection)
            .service(get_rss_feed)
            .app_data(Data::new(chat_server.clone()))
            .app_data(Data::new(Mutex::new(podcast_episode_service.clone())))
            .app_data(Data::new(Mutex::new(podcast_service.clone())))
            .app_data(Data::new(Mutex::new(db.clone())))
            .app_data(Data::new(Mutex::new(mapping_service.clone())))
            .app_data(Data::new(Mutex::new(file_service.clone())))
    })
        .bind(("0.0.0.0", 8000))?
        .run()
        .await
}


pub fn get_api_config()->Scope{
    web::scope("/api/v1")
        .service(find_podcast)
        .service(add_podcast)
        .service(find_all_podcasts)
        .service(find_all_podcast_episodes_of_podcast)
        .service(find_podcast_by_id)
        .service(log_watchtime)
        .service(get_last_watched)
        .service(get_watchtime)
        .service(get_unread_notifications)
        .service(dismiss_notifications)
        .service(download_podcast)
        .service(query_for_podcast)
        .service(download_podcast_episodes_of_podcast)
        .service(get_sys_info)
        .service(get_favored_podcasts)
        .service(favorite_podcast)
        .service(get_watchtime)
}

pub fn get_ui_config()->Scope{
    web::scope("/ui")
        .route("/index.html", web::get().to(index))
        .route("/{path:[^.]*}", web::get().to(index))
        .service(Files::new("/", "./static").index_file("index.html"))
}

pub fn get_cors_config()->Cors{
    Cors::default()
        .allow_any_origin()
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
        .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
        .allowed_header(http::header::CONTENT_TYPE)
        .allowed_header(http::header::CONNECTION)
        .allowed_header(http::header::UPGRADE)
        .max_age(3600)
}