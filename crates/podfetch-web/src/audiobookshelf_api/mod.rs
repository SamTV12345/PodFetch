//! audiobookshelf-compatible API surface.
//!
//! Wired only when `AUDIOBOOKSHELF_INTEGRATION_ENABLED=true`. Endpoints live at
//! root paths (`/login`, `/api/...`, `/public/...`, `/hls/...`, `/socket.io/`) because
//! the official audiobookshelf mobile apps hardcode these against the server base URL.
//!
//! Auth uses `users.api_key` as a Bearer token (see plan
//! `c-users-samue-webstormprojects-audiobook-reactive-puzzle.md`).
pub mod auth_middleware;
pub mod controllers;
pub mod dto;
pub mod mapping;
pub mod socket_io;

#[cfg(test)]
pub mod test_support;

#[cfg(test)]
mod tests;
