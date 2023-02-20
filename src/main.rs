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
use std::process::Command;
use std::time::Duration;
use clokwerk::{Scheduler, TimeUnits};
use feed_rs::parser;
use reqwest::blocking::{Client, ClientBuilder};
use rusqlite::Connection;
use rocket_contrib::serve::StaticFiles;
mod controllers;
pub use controllers::user_controller::*;
use crate::config::cors;
use crate::constants::constants::{DB_NAME, PODCASTS_ROOT_DIRECTORY};
use crate::models::itunes_models::Podcast;
use crate::service::rust_service::{insert_podcast_episodes, schedule_episode_download};
use crate::service::file_service::create_podcast_root_directory_exists;

mod db;
mod models;
mod constants;
mod service;
use crate::db::DB;
mod config;

fn rocket() -> rocket::Rocket {
    dotenv().ok();

    rocket::ignite()
        .attach(cors::CORS)
        .mount("/api/v1/",
               routes![get_all, new_user, find_user, find_podcast,
                   add_podcast,find_all_podcasts, find_all_podcast_episodes_of_podcast])
        .mount(PODCASTS_ROOT_DIRECTORY, StaticFiles::from("podcasts"))
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
    rocket().launch();
}