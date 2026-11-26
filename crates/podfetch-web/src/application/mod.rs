//! The composition root of PodFetch.
//!
//! [`Application`] is the one place that opens the database, wires every
//! service into an [`AppState`], builds the HTTP router and starts the
//! background jobs. Everything else gets its dependencies handed in.
//!
//! ```no_run
//! # async fn start() -> Result<(), Box<dyn std::error::Error>> {
//! use common_infrastructure::runtime::ENVIRONMENT_SERVICE;
//! use podfetch_web::application::Application;
//! use std::sync::Arc;
//!
//! let app = Application::build(Arc::new(ENVIRONMENT_SERVICE.clone()))?;
//! app.run().await?;
//! # Ok(())
//! # }
//! ```

mod jobs;

use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use common_infrastructure::config::EnvironmentService;
use common_infrastructure::error::CustomError;
use tracing::{info, warn};

use crate::app_state::AppState;
use crate::services::file::service::FileService;
use crate::startup::{build_router, insert_default_settings_if_not_present};
use jobs::BackgroundJobs;

/// How long in-flight requests get to finish after a shutdown signal.
/// Long-lived connections (socket.io, streams) would otherwise keep the
/// process alive until the container runtime kills it.
const SHUTDOWN_GRACE: Duration = Duration::from_secs(5);

#[derive(Debug, thiserror::Error)]
pub enum StartupError {
    #[error("invalid configuration: {0}")]
    Config(&'static str),
    #[error("could not create the podcast directory: {0}")]
    PodcastDirectory(CustomError),
    #[error("could not prepare the database: {0}")]
    Seed(CustomError),
}

pub struct Application {
    state: AppState,
    config: Arc<EnvironmentService>,
}

impl Application {
    /// Validates the configuration, migrates the database, wires all services
    /// and seeds what a fresh installation needs (settings, standard user).
    pub fn build(config: Arc<EnvironmentService>) -> Result<Self, StartupError> {
        validate_config(&config)?;
        podfetch_persistence::run_migrations();
        let state = AppState::build(podfetch_persistence::db::database(), config.clone());

        FileService::create_podcast_root_directory_exists()
            .map_err(StartupError::PodcastDirectory)?;
        insert_default_settings_if_not_present(&state.settings_service)
            .map_err(StartupError::Seed)?;
        state
            .user_auth_service
            .ensure_standard_user_present()
            .map_err(StartupError::Seed)?;
        if config.audiobookshelf_integration_enabled
            && let Err(e) = state.audiobookshelf_library_service.bootstrap_defaults()
        {
            tracing::error!("Could not bootstrap audiobookshelf default libraries: {e:?}");
        }

        Ok(Self { state, config })
    }

    pub fn state(&self) -> &AppState {
        &self.state
    }

    /// The complete HTTP router (API, UI, socket.io, docs), without starting
    /// any background job. Tests drive the server through this.
    pub fn router(&self) -> Router {
        build_router(self.state.clone())
    }

    /// Serves HTTP and runs the background jobs until SIGINT/SIGTERM, then
    /// stops the jobs and gives open requests [`SHUTDOWN_GRACE`] to finish.
    pub async fn run(self) -> std::io::Result<()> {
        let router = self.router();
        let jobs = BackgroundJobs::start(&self.state, &self.config);

        let bind_addr = format!("0.0.0.0:{}", self.config.port);
        let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
        info!("Listening on {bind_addr}");

        let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();
        let mut server = tokio::spawn(
            axum::serve(listener, router)
                .with_graceful_shutdown(async {
                    let _ = stop_rx.await;
                })
                .into_future(),
        );

        tokio::select! {
            result = &mut server => {
                jobs.stop(SHUTDOWN_GRACE).await;
                return result.map_err(std::io::Error::other)?;
            }
            () = shutdown_signal() => info!("Shutdown signal received, stopping PodFetch"),
        }

        let _ = stop_tx.send(());
        // Both within the same grace period: `docker stop` kills after 10 s.
        let ((), server) = tokio::join!(
            jobs.stop(SHUTDOWN_GRACE),
            tokio::time::timeout(SHUTDOWN_GRACE, server)
        );
        match server {
            Ok(result) => result.map_err(std::io::Error::other)?,
            Err(_) => {
                warn!(
                    "Open connections did not close within {}s, exiting anyway",
                    SHUTDOWN_GRACE.as_secs()
                );
                Ok(())
            }
        }
    }
}

/// Rejects configurations PodFetch cannot run with.
fn validate_config(config: &EnvironmentService) -> Result<(), StartupError> {
    if config.http_basic && (config.username.is_none() || config.password.is_none()) {
        return Err(StartupError::Config(
            "BASIC_AUTH is enabled but USERNAME or PASSWORD is not set",
        ));
    }
    let auth_methods = [
        config.http_basic,
        config.oidc_configured,
        config.reverse_proxy,
    ]
    .into_iter()
    .filter(|enabled| *enabled)
    .count();
    if config.gpodder_integration_enabled && auth_methods == 0 {
        return Err(StartupError::Config(
            "GPODDER_INTEGRATION_ENABLED requires BASIC_AUTH, OIDC_AUTH or REVERSE_PROXY",
        ));
    }
    if auth_methods > 1 {
        return Err(StartupError::Config(
            "only one of BASIC_AUTH, OIDC_AUTH and REVERSE_PROXY can be enabled",
        ));
    }
    Ok(())
}

/// Resolves on Ctrl+C, or on SIGTERM (what `docker stop` sends).
async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(e) = tokio::signal::ctrl_c().await {
            warn!("Could not listen for Ctrl+C: {e}");
            std::future::pending::<()>().await;
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(e) => {
                warn!("Could not listen for SIGTERM: {e}");
                std::future::pending::<()>().await;
            }
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> EnvironmentService {
        let mut config = common_infrastructure::runtime::ENVIRONMENT_SERVICE.clone();
        config.http_basic = false;
        config.oidc_configured = false;
        config.reverse_proxy = false;
        config.gpodder_integration_enabled = false;
        config
    }

    #[test]
    fn accepts_a_plain_configuration() {
        assert!(validate_config(&config()).is_ok());
    }

    #[test]
    fn basic_auth_needs_credentials() {
        let mut config = config();
        config.http_basic = true;
        config.username = None;
        assert!(matches!(
            validate_config(&config),
            Err(StartupError::Config(_))
        ));
    }

    #[test]
    fn gpodder_needs_an_auth_method() {
        let mut config = config();
        config.gpodder_integration_enabled = true;
        assert!(validate_config(&config).is_err());
        config.reverse_proxy = true;
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn only_one_auth_method() {
        let mut config = config();
        config.reverse_proxy = true;
        config.oidc_configured = true;
        assert!(validate_config(&config).is_err());
    }
}
