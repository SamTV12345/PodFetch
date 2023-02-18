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
use std::env;
use std::process::Command;
use rusqlite::Connection;

mod controllers;
pub use controllers::user_controller::*;
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
    let conn = Connection::open("cats.db");

    let connre = conn.unwrap();

       connre.execute("create table if not exists podcast (
             id integer primary key,
             name text not null unique,
             directory text not null,
             rssfeed text not null)", []).expect("Error creating table");
    rocket().launch();
}