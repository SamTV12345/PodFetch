//! gPodder sync bounded context: devices, sync groups, subscriptions and
//! the gPodder settings of each user.
//!
//! The episode actions of the gPodder API belong to the listening context;
//! the gPodder controllers in the web layer combine both.

pub mod domain;
pub mod dto;
pub mod events;
pub mod handlers;
pub mod persistence;
pub mod services;
pub mod usecases;
