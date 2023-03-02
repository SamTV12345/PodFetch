#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::{thread};
use std::env::var;
use std::error::Error;
use actix_web::{App, http, HttpResponse, HttpServer, Responder, web};
use std::time::Duration;
use actix_cors::Cors;
use actix_files::Files;
use actix_web::middleware::Logger;
use clokwerk::{Scheduler, TimeUnits};
use diesel::QueryDsl;

mod controllers;
pub use controllers::user_controller::*;
use crate::config::DBConfig::establish_connection;
use crate::service::rust_service::{insert_podcast_episodes, schedule_episode_download};
use crate::service::file_service::create_podcast_root_directory_exists;
mod db;
mod models;
mod constants;
mod service;
use crate::db::DB;
use crate::models::itunes_models::Podcast;
use crate::service::environment_service::EnvironmentService;
use crate::service::logging_service::init_logging;
use diesel::prelude::*;

mod config;
pub mod schema;

use crate::schema::podcasts::dsl::podcasts;

pub fn run_poll(){
    let mut db = DB::new().unwrap();
    //check for new episodes
    let podcats_result = db.get_podcasts().unwrap();
    for podcast in podcats_result {
    let podcast_clone = podcast.clone();
    insert_podcast_episodes(podcast);
    schedule_episode_download(podcast_clone);
}
}

async fn index() ->  impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(include_str!("../static/index.html"))
}

#[actix_web::main]
async fn main()-> std::io::Result<()> {

    let conn = &mut establish_connection();


    EnvironmentService::print_banner();
    init_logging();
    DB::new().unwrap();
    create_podcast_root_directory_exists();

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


    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);

        let ui = web::scope("/ui")
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
            .service(get_watchtime);

           // .wrap(Logger::default());
        App::new().service(Files::new
            ("/podcasts", "podcasts").show_files_listing())
            .wrap(cors)
            .service(api)
            .service(ui)
            //.wrap(Logger::default())
    }
    )
        .bind(("0.0.0.0", 8000))?
        .run()
        .await
}