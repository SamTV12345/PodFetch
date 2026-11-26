//! Background jobs started by [`super::Application::run`].

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use clokwerk::{Scheduler, TimeUnits};
use common_infrastructure::config::EnvironmentService;
use tracing::info;

use crate::app_state::AppState;
use crate::startup::run_poll;
use crate::usecases::podcast_episode::PodcastEpisodeUseCase;

/// Handles to everything [`BackgroundJobs::start`] spawned.
pub(super) struct BackgroundJobs {
    scheduler_running: Arc<AtomicBool>,
    transcription_worker: Option<tokio::task::JoinHandle<()>>,
}

impl BackgroundJobs {
    pub(super) fn start(state: &AppState, config: &EnvironmentService) -> Self {
        let scheduler_running = Arc::new(AtomicBool::new(true));
        spawn_scheduler(state, config, scheduler_running.clone());

        // Gated on a configured backend, although the worker itself no-ops
        // without one, so no task exists at all when the feature is off.
        let transcription_worker = config
            .transcription_config
            .is_some()
            .then(|| tokio::spawn(crate::services::transcript::worker::run_transcription_worker()));

        if let Some(oidc_config) = config.oidc_config.as_ref() {
            crate::services::oidc::discovery::spawn_endpoint_discovery(oidc_config);
        }

        Self {
            scheduler_running,
            transcription_worker,
        }
    }

    /// Signals all jobs to stop. A scheduler job that is running right now
    /// finishes its current run; nothing new is started afterwards.
    pub(super) fn stop(self) {
        self.scheduler_running.store(false, Ordering::Relaxed);
        if let Some(worker) = self.transcription_worker {
            worker.abort();
        }
        info!("Background jobs stopped");
    }
}

/// Polls feeds every polling interval and cleans up once a day.
fn spawn_scheduler(state: &AppState, config: &EnvironmentService, running: Arc<AtomicBool>) {
    let settings_for_polling = state.settings_service.clone();
    let settings_for_cleanup = state.settings_service.clone();
    let sessions = state.session_service.clone();
    let polling_interval = config.get_polling_interval();

    let spawned = thread::Builder::new()
        .name("scheduler".to_owned())
        .spawn(move || {
            let mut scheduler = Scheduler::new();

            scheduler.every(polling_interval.minutes()).run(move || {
                // A panic here would silently stop polling until the next restart.
                match settings_for_polling.get_settings() {
                    Ok(Some(settings)) if settings.auto_update => {
                        info!("Polling for new episodes");
                        match run_poll() {
                            Ok(()) => info!("Polling for new episodes successful"),
                            Err(e) => tracing::error!("Error polling for new episodes: {e}"),
                        }
                    }
                    Ok(Some(_)) => {}
                    Ok(None) => tracing::error!("Could not get settings from database"),
                    Err(e) => tracing::error!("Could not load settings for polling: {e}"),
                }
            });

            scheduler.every(1.day()).run(move || {
                if let Err(e) = sessions.cleanup_expired() {
                    tracing::error!("Error clearing old sessions: {e}");
                }
                match settings_for_cleanup.get_settings() {
                    Ok(Some(settings)) if settings.auto_cleanup => {
                        PodcastEpisodeUseCase::cleanup_old_episodes(settings.auto_cleanup_days);
                    }
                    Ok(Some(_)) => {}
                    Ok(None) => tracing::error!("Could not get settings from database"),
                    Err(e) => tracing::error!("Could not load settings for cleanup: {e}"),
                }
            });

            while running.load(Ordering::Relaxed) {
                scheduler.run_pending();
                thread::sleep(Duration::from_secs(1));
            }
        });
    if let Err(e) = spawned {
        tracing::error!("Could not start the scheduler thread: {e}");
    }
}
