use diesel_migrations::{EmbeddedMigrations, embed_migrations};

#[macro_use]
extern crate serde_derive;
extern crate core;
extern crate serde_json;

use std::env;
use std::env::args;
use std::process::exit;

use crate::command_line_runner::start_command_line;
use crate::commands::startup::handle_config_for_server_startup;

use common_infrastructure::config::EnvironmentService;
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;

mod command_line_runner;
mod commands;
mod test_utils;

macro_rules! import_database_config {
    () => {
        #[cfg(feature = "sqlite")]
        pub const SQLITE_MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations/sqlite");

        #[cfg(feature = "postgresql")]
        pub const POSTGRES_MIGRATIONS: EmbeddedMigrations =
            embed_migrations!("./migrations/postgres");
    };
}

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
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;

    axum::serve(listener, router).await?;
    Ok(())
}
