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
