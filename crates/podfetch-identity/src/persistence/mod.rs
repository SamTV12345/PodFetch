//! Diesel repositories for the tables this context owns:
//! `users`, `sessions` and `invites`.

pub mod adapters;
pub mod invite;
pub mod session;
pub mod user_admin;
