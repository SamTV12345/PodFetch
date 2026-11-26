use std::env;
use std::env::args;
use std::process::exit;

use crate::command_line_runner::start_command_line;

use common_infrastructure::config::EnvironmentService;
use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
use podfetch_web::application::Application;
use std::sync::Arc;

mod agent;
mod command_line_runner;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!(
        "Debug file located at location {}",
        concat!(env!("OUT_DIR"), "/built.rs")
    );

    EnvironmentService::print_banner();
    ENVIRONMENT_SERVICE.get_environment();

    let raw_args: Vec<String> = args().collect();
    if raw_args.iter().any(|a| a == "--agent") {
        let mut iter = raw_args.into_iter();
        iter.next(); // skip binary path
        let agent_args: Vec<String> = iter.collect();
        let config = match agent::config::parse_from_iter(agent_args) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("agent config error: {e}");
                exit(2);
            }
        };
        return agent::run_agent(config).await;
    }

    let app = match Application::build(Arc::new(ENVIRONMENT_SERVICE.clone())) {
        Ok(app) => app,
        Err(e) => {
            eprintln!("PodFetch cannot start: {e}");
            exit(1);
        }
    };

    if args().len() > 1 {
        if let Err(e) = start_command_line(app.state(), args()).await {
            tracing::error!("Error in command line: {e}");
            exit(1);
        }
        exit(0)
    }

    app.run().await
}
