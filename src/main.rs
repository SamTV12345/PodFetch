use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::{env, thread};
use actix_web::{App, http, HttpResponse, HttpServer, Responder, web};
use std::time::Duration;
use actix::Actor;
use actix_cors::Cors;
use actix_files::Files;
use actix_web::middleware::Logger;
use actix_web::web::{redirect};
use clokwerk::{Scheduler, TimeUnits};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod controllers;
pub use controllers::controller_utils::*;
use crate::config::dbconfig::establish_connection;
use crate::controllers::api_doc::ApiDoc;
use crate::controllers::podcast_controller::{download_podcast, query_for_podcast};
use crate::controllers::notification_controller::{dismiss_notifications, get_unread_notifications};
use crate::controllers::podcast_controller::{add_podcast, find_all_podcasts, find_podcast, find_podcast_by_id};
use crate::controllers::podcast_episode_controller::{download_podcast_episodes_of_podcast, find_all_podcast_episodes_of_podcast};
use crate::controllers::watch_time_controller::{get_last_watched, get_watchtime, log_watchtime};
use crate::controllers::websocket_controller::{send_message_to_user, start_connection};
use crate::service::rust_service::{schedule_episode_download};
mod db;
mod models;
mod constants;
mod service;
use crate::db::DB;
use crate::models::web_socket_message::Lobby;
use crate::service::environment_service::EnvironmentService;
use crate::service::file_service::FileService;
use crate::service::logging_service::init_logging;
use crate::service::podcast_episode_service::PodcastEpisodeService;

mod config;
pub mod schema;


pub fn run_poll(){
    let mut db = DB::new().unwrap();
    //check for new episodes
    let podcats_result = db.get_podcasts().unwrap();
    for podcast in podcats_result {
    let podcast_clone = podcast.clone();
    PodcastEpisodeService::insert_podcast_episodes(podcast);
    schedule_episode_download(podcast_clone);
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
    println!("cargo:rerun-if-changed=./Cargo.toml");

    let lobby = Lobby::default();

    let chat_server = lobby.start();

    let mut connection = establish_connection();
    connection.run_pending_migrations(MIGRATIONS).unwrap();

    EnvironmentService::print_banner();
    init_logging();
    DB::new().unwrap();
    FileService::create_podcast_root_directory_exists();

    thread::spawn(||{
        let mut scheduler = Scheduler::new();
        let env = EnvironmentService::new();
        env.get_environment();
        let polling_interval = env.get_polling_interval();
        scheduler.every(polling_interval.minutes()).run(||{
            log::info!("Polling for new episodes");
           run_poll();
        });
        loop {
            scheduler.run_pending();
            thread::sleep(Duration::from_millis(1000));
        }
    });


    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .allowed_header(http::header::CONNECTION)
            .allowed_header(http::header::UPGRADE)
            .max_age(3600);

        let ui = web::scope("/ui")
            .route("/index.html", web::get().to(index))
            .route("/{path:[^.]*}", web::get().to(index))
            .service(Files::new("/", "./static").index_file("index.html"));

        let api = web::scope("/api/v1")
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
            .service(get_watchtime);

        let openapi = ApiDoc::openapi();

           // .wrap(Logger::default());
        App::new().service(Files::new
            ("/podcasts", "podcasts").show_files_listing())
            .service(redirect("/swagger-ui", "/swagger-ui/"))
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-doc/openapi.json", openapi),
            )
            .service(redirect("/","/ui/"))
            .wrap(cors)
            .service(api)
            .service(ui)
            .service(start_connection)
            .service(send_message_to_user)
            .data(chat_server.clone())
            .wrap(Logger::default())
    }
    )
        .bind(("0.0.0.0", 8000))?
        .run()
        .await
}