//! The identity model. Private to the context in spirit: other contexts use
//! [`crate::api`] and refer to users by [`podfetch_kernel::UserId`].

pub mod invite;
pub mod session;
pub mod user;
pub mod user_admin;
