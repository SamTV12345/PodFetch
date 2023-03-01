#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::{thread};
use std::env::var;
use actix_files as fs;
use actix_web::{App, http, HttpServer, web};
use std::time::Duration;
use actix_cors::Cors;
use actix_web::middleware::Logger;
use clokwerk::{Scheduler, TimeUnits};
mod controllers;
pub use controllers::user_controller::*;
use crate::service::rust_service::{insert_podcast_episodes, schedule_episode_download};
use crate::service::file_service::create_podcast_root_directory_exists;
mod db;
mod models;
mod constants;
mod service;
use crate::db::DB;
use crate::service::environment_service::EnvironmentService;
use crate::service::logging_service::init_logging;

mod config;

pub fn run_poll(){
    let db = DB::new().unwrap();
    //check for new episodes
    let podcasts = db.get_podcasts().unwrap();
    for podcast in podcasts {
    let podcast_clone = podcast.clone();
    insert_podcast_episodes(podcast);
    schedule_episode_download(podcast_clone);
}
}


#[actix_web::main]
async fn main()-> std::io::Result<()> {
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
           run_poll();
        });
        loop {
            scheduler.run_pending();
            thread::sleep(Duration::from_millis(1000));
        }
    });

    // Start WebSocket server
    /*let websocket_server = thread::spawn(||HttpServer::new(|| {
        App::new()
            .service(web::resource("/ws/"))
    })
        .bind("127.0.0.1:8080")
        .unwrap()
        .run());*/


    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);


        let api = web::scope("/api/v1")
            .service(find_podcast)
            .service(add_podcast)
            .service(find_all_podcasts)
            .service(find_all_podcast_episodes_of_podcast)
            .service(find_podcast_by_id)
            .service(log_watchtime)
            .service(get_last_watched)
            .service(get_watchtime)

            .wrap(Logger::default());
        App::new().service(fs::Files::new
            ("/podcasts", "podcasts").show_files_listing())
            .service(fs::Files::new("/ui", "./static"))
            .wrap(cors)
            .service(api)
            .wrap(Logger::default())
    }
    )
        .bind(("0.0.0.0", 8000))?
        .run()
        .await
}