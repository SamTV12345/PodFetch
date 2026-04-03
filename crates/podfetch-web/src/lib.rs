// Existing DTOs, traits, and types
pub mod auth;
pub mod device;
pub mod events;
pub mod file_access;
pub mod filter;
pub mod gpodder;
pub mod history;
pub mod invite;
pub mod manifest;
pub mod notification;
pub mod playlist;
pub mod podcast;
pub mod podcast_episode;
pub mod podcast_episode_dto;
pub mod podcast_settings;
pub mod podcast_view;
pub mod role;
pub mod rss;
pub mod settings;
pub mod stats;
pub mod subscription;
pub mod sys;
pub mod tags;
pub mod url_rewriting;
pub mod user_admin;
pub mod user_onboarding;
pub mod watchtime;

// Application services (implementations of the traits above)
pub mod services;

// Use cases
pub mod usecases;

// Repository abstractions
pub mod repositories;

// HTTP controllers (Axum handlers)
pub mod controllers;

// WebSocket / ChatServerHandle
pub mod server;

// Authentication middleware
pub mod auth_middleware;

// GPodder API handlers
pub mod gpodder_api;

// Route configuration
pub mod routes;

// Server startup, scheduling, UI serving
pub mod startup;

// Application state
pub mod app_state;

// File access API adapter
pub mod api_file_access;

// Test utilities
#[cfg(test)]
pub mod test_utils;
#[cfg(test)]
pub mod test_support;

/// Runs before any test, ensuring env vars are set before
/// `ENVIRONMENT_SERVICE` LazyLock is first accessed.
#[cfg(test)]
#[ctor::ctor]
fn init_test_env() {
    // SAFETY: This runs before any test thread starts.
    unsafe {
        std::env::set_var("BASIC_AUTH", "true");
        std::env::set_var("USERNAME", "postgres");
        std::env::set_var("PASSWORD", "postgres");
        std::env::set_var("GPODDER_INTEGRATION_ENABLED", "true");
        std::env::set_var("API_KEY", "test-api-key");
        #[cfg(feature = "sqlite")]
        std::env::set_var("DATABASE_URL", "sqlite://./podcast.db");
        #[cfg(all(feature = "postgresql", not(feature = "sqlite")))]
        std::env::set_var(
            "DATABASE_URL",
            "postgres://postgres:postgres@127.0.0.1:55002/postgres",
        );
    }
}

