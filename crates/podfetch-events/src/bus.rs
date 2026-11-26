use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, PoisonError, RwLock};
use std::time::Duration;

use chrono::{DateTime, Utc};
use common_infrastructure::db::{DBType, Database};
use diesel::Connection;
use futures::FutureExt;
use futures::future::BoxFuture;
use tokio::sync::Notify;
use uuid::Uuid;

use crate::event::{Event, EventMeta, HandlerError};
use crate::store::{self, DeliveryStatus, DeliveryView, EventBusError, NewEvent, StoredEvent};

/// Tuning knobs of the processor; the defaults suit production.
#[derive(Debug, Clone)]
pub struct EventBusConfig {
    /// Fallback poll when no publish wakes the processor (e.g. a retry
    /// becoming due, or a publish from another process).
    pub poll_interval: Duration,
    /// Deliveries claimed per round.
    pub batch_size: i64,
    /// Deliveries processed at the same time.
    pub concurrency: usize,
    /// A handler running longer counts as failed.
    pub handler_timeout: Duration,
    /// Attempts before a delivery is `dead`.
    pub max_attempts: i32,
    /// Wait before retry n (the last value repeats).
    pub backoff: Vec<Duration>,
    /// Completed deliveries older than this are deleted.
    pub retention: Duration,
}

impl Default for EventBusConfig {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_secs(5),
            batch_size: 32,
            concurrency: 4,
            handler_timeout: Duration::from_secs(5 * 60),
            max_attempts: 10,
            backoff: [5, 30, 120, 600, 3600]
                .into_iter()
                .map(Duration::from_secs)
                .collect(),
            retention: Duration::from_secs(30 * 24 * 3600),
        }
    }
}

impl EventBusConfig {
    pub(crate) fn backoff_for(&self, attempt: i32) -> Duration {
        let index = usize::try_from(attempt.max(1) - 1).unwrap_or(0);
        self.backoff
            .get(index)
            .or_else(|| self.backoff.last())
            .copied()
            .unwrap_or(Duration::from_secs(60))
    }

    /// How long a claimed delivery stays leased: long enough for a slow
    /// handler, short enough that a crashed one is retried soon.
    pub(crate) fn lease(&self) -> Duration {
        self.handler_timeout + Duration::from_secs(60)
    }
}

pub(crate) type HandlerFn =
    Arc<dyn Fn(StoredEvent, i32) -> BoxFuture<'static, Result<(), HandlerError>> + Send + Sync>;

pub struct EventBus {
    pub(crate) database: Database,
    pub(crate) config: EventBusConfig,
    /// event type → (handler name, handler)
    handlers: RwLock<HashMap<&'static str, Vec<(&'static str, HandlerFn)>>>,
    pub(crate) wake: Notify,
}

impl EventBus {
    pub fn new(database: Database, config: EventBusConfig) -> Arc<Self> {
        Arc::new(Self {
            database,
            config,
            handlers: RwLock::new(HashMap::new()),
            wake: Notify::new(),
        })
    }

    /// Registers `handler` under the stable name `name` for events of type `E`.
    ///
    /// Register every handler before the first publish of that type: a
    /// delivery is only created for handlers known at publish time. The name
    /// is stored with each delivery, so keep it stable across releases.
    ///
    /// # Panics
    /// When `name` is already registered for `E`, which is a wiring bug.
    pub fn subscribe<E, F, Fut>(&self, name: &'static str, handler: F)
    where
        E: Event,
        F: Fn(E, EventMeta) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), HandlerError>> + Send + 'static,
    {
        let handler = Arc::new(handler);
        let erased: HandlerFn = Arc::new(move |stored: StoredEvent, attempt: i32| {
            match decode::<E>(&stored, attempt) {
                Ok((event, meta)) => handler(event, meta).boxed(),
                Err(error) => futures::future::ready(Err(error)).boxed(),
            }
        });
        let mut handlers = self
            .handlers
            .write()
            .unwrap_or_else(PoisonError::into_inner);
        let registered = handlers.entry(E::TYPE).or_default();
        assert!(
            registered.iter().all(|(existing, _)| *existing != name),
            "handler {name} is subscribed to {} twice",
            E::TYPE
        );
        registered.push((name, erased));
    }

    pub(crate) fn handler(&self, event_type: &str, name: &str) -> Option<HandlerFn> {
        self.handlers
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(event_type)?
            .iter()
            .find(|(registered, _)| *registered == name)
            .map(|(_, handler)| handler.clone())
    }

    fn handler_names(&self, event_type: &str) -> Vec<&'static str> {
        self.handlers
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(event_type)
            .map_or_default(|handlers| handlers.iter().map(|(name, _)| *name).collect())
    }

    /// Runs `work` in one database transaction. Events published through
    /// [`Tx::publish`] are stored only if the transaction commits, and the
    /// processor is woken right after the commit.
    pub fn transaction<T, E>(&self, work: impl FnOnce(&mut Tx<'_>) -> Result<T, E>) -> Result<T, E>
    where
        E: From<EventBusError>,
    {
        enum Failure<E> {
            Database(diesel::result::Error),
            Work(E),
        }
        impl<E> From<diesel::result::Error> for Failure<E> {
            fn from(error: diesel::result::Error) -> Self {
                Self::Database(error)
            }
        }

        let mut conn = self
            .database
            .connection()
            .map_err(|e| E::from(EventBusError::Connection(e)))?;
        let mut published = false;
        let result = conn.transaction(|conn| {
            let mut tx = Tx {
                conn,
                bus: self,
                published: &mut published,
            };
            work(&mut tx).map_err(Failure::Work)
        });
        match result {
            Ok(value) => {
                if published {
                    self.wake.notify_one();
                }
                Ok(value)
            }
            Err(Failure::Work(error)) => Err(error),
            Err(Failure::Database(error)) => Err(E::from(EventBusError::Database(error))),
        }
    }

    /// Publishes a single event in its own transaction.
    pub fn publish<E: Event>(&self, event: &E) -> Result<(), EventBusError> {
        self.transaction(|tx| tx.publish(event))
    }

    /// Deliveries for the admin view, newest first.
    pub fn deliveries(
        &self,
        status: Option<DeliveryStatus>,
        limit: i64,
    ) -> Result<Vec<DeliveryView>, EventBusError> {
        store::deliveries(&self.database, status, limit)
    }

    /// Number of deliveries per status.
    pub fn counts(&self) -> Result<Vec<(String, i64)>, EventBusError> {
        store::counts(&self.database)
    }

    /// Re-queues a failed or dead delivery and wakes the processor.
    pub fn retry(&self, event_seq: i64, handler: &str) -> Result<(), EventBusError> {
        store::retry(&self.database, event_seq, handler)?;
        self.wake.notify_one();
        Ok(())
    }
}

/// A running transaction of [`EventBus::transaction`].
pub struct Tx<'a> {
    conn: &'a mut DBType,
    bus: &'a EventBus,
    published: &'a mut bool,
}

impl Tx<'_> {
    /// The transaction's connection, for repositories taking part in it.
    pub fn conn(&mut self) -> &mut DBType {
        self.conn
    }

    pub fn publish<E: Event>(&mut self, event: &E) -> Result<(), EventBusError> {
        self.publish_correlated(event, None)
    }

    /// Publishes `event` as a consequence of another event (`correlation_id`
    /// = the id that started the chain), so the chain can be traced.
    pub fn publish_correlated<E: Event>(
        &mut self,
        event: &E,
        correlation_id: Option<Uuid>,
    ) -> Result<(), EventBusError> {
        let payload = serde_json::to_string(event).map_err(|source| EventBusError::Serialize {
            event_type: E::TYPE,
            source,
        })?;
        let new_event = NewEvent {
            id: Uuid::now_v7().to_string(),
            event_type: E::TYPE,
            version: E::VERSION,
            aggregate_id: event.aggregate_id(),
            payload,
            occurred_at: store::now(),
            correlation_id: correlation_id.map(|id| id.to_string()),
        };
        store::append(self.conn, &new_event, &self.bus.handler_names(E::TYPE))?;
        *self.published = true;
        Ok(())
    }
}

/// Turns a stored row into a typed event; a failure here is permanent.
fn decode<E: Event>(stored: &StoredEvent, attempt: i32) -> Result<(E, EventMeta), HandlerError> {
    let raw: serde_json::Value = serde_json::from_str(&stored.payload)
        .map_err(|e| HandlerError::permanent(format!("stored payload is not JSON: {e}")))?;
    let current = E::upcast(stored.event_version, raw).map_err(HandlerError::permanent)?;
    let event = serde_json::from_value(current).map_err(|e| {
        HandlerError::permanent(format!(
            "payload does not match {} v{}: {e}",
            E::TYPE,
            E::VERSION
        ))
    })?;
    let meta = EventMeta {
        id: Uuid::parse_str(&stored.id)
            .map_err(|e| HandlerError::permanent(format!("invalid event id: {e}")))?,
        seq: stored.seq,
        aggregate_id: stored.aggregate_id.clone(),
        occurred_at: DateTime::<Utc>::from_naive_utc_and_offset(stored.occurred_at, Utc),
        correlation_id: stored
            .correlation_id
            .as_deref()
            .and_then(|id| Uuid::parse_str(id).ok()),
        attempt,
    };
    Ok((event, meta))
}
