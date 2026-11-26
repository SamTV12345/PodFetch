//! Identity & Access: users, roles, sessions, invites and login.
//!
//! Other contexts talk to identity only through [`api`] (synchronous
//! questions such as "who owns this API key?") and [`events`] (facts such as
//! [`events::UserDeleted`]), and refer to users by [`podfetch_kernel::UserId`].

pub mod api;
pub mod domain;
pub mod events;
pub mod invite;
pub mod login;
pub mod persistence;
pub mod role;
pub mod services;
pub mod user_admin;
pub mod user_onboarding;
