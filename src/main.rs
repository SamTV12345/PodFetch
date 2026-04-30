use std::env;
use std::env::args;
use std::process::exit;

use crate::command_line_runner::start_command_line;

use common_infrastructure::config::EnvironmentService;
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
use podfetch_web::startup::handle_config_for_server_startup;

mod command_line_runner;

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
            tracing::error!("Error in command line: {e}");
            exit(1);
        }
        exit(0)
    }

    let router = handle_config_for_server_startup();
    let port = ENVIRONMENT_SERVICE.port;
    let bind_addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!("Listening on {bind_addr}");

    axum::serve(listener, router).await?;
    Ok(())
}
