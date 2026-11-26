//! A database-backed event bus.
//!
//! Events are written to the `events` table in the same transaction as the
//! change that caused them (transactional outbox), so an event is never lost
//! and never published for a change that was rolled back. For every handler
//! subscribed to the event's type, a row in `event_deliveries` tracks that
//! handler's progress on its own: a failing handler is retried with
//! exponential backoff and ends up as `dead` after too many attempts, without
//! affecting the other handlers of the same event.
//!
//! ```ignore
//! #[derive(Serialize, Deserialize)]
//! struct EpisodeDiscovered { episode_id: Uuid, podcast_id: Uuid }
//!
//! impl Event for EpisodeDiscovered {
//!     const TYPE: &'static str = "catalog.episode_discovered";
//!     fn aggregate_id(&self) -> String { self.podcast_id.to_string() }
//! }
//!
//! bus.subscribe("media.auto_download", |event: EpisodeDiscovered, _meta| async move {
//!     download(event.episode_id).await?;
//!     Ok(())
//! });
//!
//! bus.transaction(|tx| {
//!     episodes.insert(tx.conn(), &episode)?;
//!     tx.publish(&EpisodeDiscovered { episode_id, podcast_id })
//! })?;
//! ```
//!
//! Delivery is at-least-once: a handler may see an event again after a crash
//! between doing its work and recording success, so handlers must be
//! idempotent. Deliveries of one handler are processed in order per aggregate.

mod bus;
mod event;
mod processor;
mod store;

pub use bus::{EventBus, EventBusConfig, Tx};
pub use event::{Event, EventMeta, HandlerError};
pub use processor::ProcessorHandle;
pub use store::{DeliveryStatus, DeliveryView, EventBusError};

#[cfg(test)]
mod tests;
