#![feature(plugin, decl_macro, proc_macro_hygiene)]
#![allow(proc_macro_derive_resolution_fallback, unused_attributes)]

#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

use dotenv::dotenv;
use std::{env, thread};
use actix_files as fs;
use actix_web::{App, HttpServer, web};
use std::process::Command;
use std::time::Duration;
use actix_web::middleware::Logger;
use clokwerk::{Scheduler, TimeUnits};
use feed_rs::parser;
use reqwest::blocking::{Client, ClientBuilder};
use rocket::{Rocket};
use rocket::http::Method;
use rusqlite::Connection;
use rocket_contrib::serve::StaticFiles;
use rocket_cors::AllowedHeaders;

mod controllers;
pub use controllers::user_controller::*;
use crate::config::cors;
use crate::config::cors::CORS;
use crate::constants::constants::{DB_NAME, PODCASTS_ROOT_DIRECTORY};
use crate::models::itunes_models::Podcast;
use crate::service::rust_service::{insert_podcast_episodes, schedule_episode_download};
use crate::service::file_service::create_podcast_root_directory_exists;
use crate::controllers::file_controller::files;
use crate::controllers::file_controller::static_rocket_route_info_for_files;
mod db;
mod models;
mod constants;
mod service;
use crate::db::DB;
mod config;

#[actix_web::main]
async fn actix() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(fs::Files::new
        ("/podcasts", "podcasts").show_files_listing()))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}


fn rocket () -> Rocket {
    dotenv().ok();

    // You can also deserialize this
    let cors = rocket_cors::CorsOptions::default().to_cors().unwrap();

    rocket::ignite()
        .attach(CORS)
        .mount("/api/v1/",
               routes![get_all, new_user, find_user, find_podcast,
                   add_podcast,find_all_podcasts, find_all_podcast_episodes_of_podcast,
                   manual_options, find_podcast_by_id])
        //.mount(PODCASTS_ROOT_DIRECTORY, routes![files])
        .manage(cors)
}

fn main() {
    DB::new().unwrap();
    create_podcast_root_directory_exists();

    thread::spawn(||{
        let mut scheduler = Scheduler::new();

        scheduler.every(300.minutes()).run(||{
            let db = DB::new().unwrap();
            //check for new episodes
            let podcasts = db.get_podcasts().unwrap();
            println!("Checking for new episodes: {:?}", podcasts);
            for podcast in podcasts {
                let podcast_clone = podcast.clone();
                insert_podcast_episodes(podcast);
                schedule_episode_download(podcast_clone)
            }
        });
        loop {
            scheduler.run_pending();
            thread::sleep(Duration::from_millis(1000));
        }
    });
    thread::spawn(||{
        actix();
    });
    rocket().launch();
}