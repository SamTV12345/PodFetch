use diesel_migrations::{EmbeddedMigrations, embed_migrations};

#[macro_use]
extern crate serde_derive;
extern crate core;
extern crate serde_json;

use std::env;
use std::env::args;
use std::process::exit;
mod controllers;
use crate::adapters::persistence::dbconfig::DBType;
use crate::adapters::persistence::dbconfig::db::get_connection;
use crate::command_line_runner::start_command_line;
use crate::commands::startup::handle_config_for_server_startup;
use crate::constants::inner_constants::ENVIRONMENT_SERVICE;
pub use controllers::controller_utils::*;

mod constants;
mod db;
mod models;
mod service;

use crate::service::environment_service::EnvironmentService;

mod config;

mod adapters;
mod application;
mod auth_middleware;
mod command_line_runner;
mod commands;
mod domain;
mod exception;
mod gpodder;
pub mod mutex;
mod test_utils;
pub mod utils;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!(
        "Debug file located at {}",
        concat!(env!("OUT_DIR"), "/built.rs")
    );

    EnvironmentService::print_banner();
    ENVIRONMENT_SERVICE.get_environment();
    if args().len() > 1 {
        if let Err(e) = start_command_line(args()).await {
            log::error!("Error in command line: {e}");
            exit(1);
        }
        exit(0)
    }

    let router = handle_config_for_server_startup();
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;

    axum::serve(listener, router).await?;
    Ok(())
}
