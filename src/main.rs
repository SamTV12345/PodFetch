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

mod controllers;
pub use controllers::user_controller::*;
use crate::constants::constants::DB_NAME;
use crate::models::itunes_models::Podcast;
use crate::service::rust_service::insert_podcast_episodes;
use crate::service::file_service::create_podcast_root_directory_exists;

mod db;
mod models;
mod constants;
mod service;

fn rocket() -> rocket::Rocket {
    dotenv().ok();

    rocket::ignite()
        .mount(
            "/api/v1/",
            routes![get_all, new_user, find_user, find_podcast, add_podcast],
        )
}

fn main() {
    init_db();
    create_podcast_root_directory_exists();

    thread::spawn(||{
        let mut scheduler = Scheduler::new();

        scheduler.every(1.minutes()).run(||{
            let connection = Connection::open(DB_NAME);
            let connection_client = connection.unwrap();
            //check for new episodes
            let mut result = connection_client.prepare("select * from Podcast")
                .expect("Error getting podcasts from database");

            let result = result.query_map([], |row| {
                Ok(Podcast {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    directory: row.get(2)?,
                    rssfeed: row.get(3)?,
                })
            }).expect("Error getting podcasts from database");

            for res in result {
                let podcast = res.unwrap();
                insert_podcast_episodes(podcast);
            }
        });
        loop {
            println!("Running scheduler...");
            scheduler.run_pending();
            thread::sleep(Duration::from_millis(1000));
        }
    });
    rocket().launch();
}

fn init_db() {
    let conn = Connection::open(DB_NAME);
    let connre = conn.unwrap();

    connre.execute("create table if not exists Podcast (
             id integer primary key,
             name text not null unique,
             directory text not null,
             rssfeed text not null)", []).expect("Error creating table");
    connre.execute("create table if not exists podcast_episodes (
             id integer primary key,
             podcast_id integer not null,
             episode_id TEXT not null,
             name text not null,
             url text not null,
             date text not null,
             FOREIGN KEY (podcast_id) REFERENCES Podcast(id))", []).expect("Error creating table");
    // status 0 = not downloaded, 1 = downloaded, 2 = error
    connre.execute("create table if not exists queue (
             id integer primary key,
             podcast_id integer not null,
             download_url text not null,
             episode_id TEXT not null,
             status integer not null,
             FOREIGN KEY (podcast_id) REFERENCES Podcast(id),
             FOREIGN KEY (episode_id) REFERENCES podcast_episodes(id))",
                   []).expect("Error creating table");
    connre.close().expect("Error closing connection");
}