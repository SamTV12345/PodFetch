//! Works through due deliveries until stopped.

use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{Semaphore, watch};
use tokio::task::JoinSet;

use crate::bus::EventBus;
use crate::store::{self, Claimed, DeliveryStatus, EventBusError};

/// How often completed deliveries past the retention are purged.
const PURGE_EVERY: Duration = Duration::from_secs(3600);

/// Returned by [`EventBus::spawn_processor`]; stops the processor on demand.
pub struct ProcessorHandle {
    stop: watch::Sender<bool>,
    task: tokio::task::JoinHandle<()>,
}

impl ProcessorHandle {
    /// Stops claiming new deliveries and waits up to `grace` for running
    /// handlers. Handlers still running afterwards are cancelled; their
    /// lease expires and they are retried after the next start.
    pub async fn stop(self, grace: Duration) {
        let _ = self.stop.send(true);
        if tokio::time::timeout(grace, self.task).await.is_err() {
            tracing::warn!("Event processor did not stop within {}s", grace.as_secs());
        }
    }
}

impl EventBus {
    /// Starts processing deliveries on the current tokio runtime.
    pub fn spawn_processor(self: &Arc<Self>) -> ProcessorHandle {
        let (stop, stopped) = watch::channel(false);
        let task = tokio::spawn(self.clone().run(stopped));
        ProcessorHandle { stop, task }
    }

    async fn run(self: Arc<Self>, mut stopped: watch::Receiver<bool>) {
        let permits = Arc::new(Semaphore::new(self.config.concurrency.max(1)));
        let mut running = JoinSet::new();
        let mut last_purge = Instant::now();

        while !*stopped.borrow() {
            while running.try_join_next().is_some() {}

            let claimable = i64::try_from(permits.available_permits())
                .unwrap_or(0)
                .min(self.config.batch_size);
            let claimed = if claimable > 0 {
                self.blocking(move |bus| {
                    let lease = chrono::Duration::from_std(bus.config.lease())
                        .unwrap_or(chrono::Duration::minutes(6));
                    store::claim(&bus.database, claimable, store::now() + lease)
                })
                .await
                .unwrap_or_else(|e| {
                    tracing::error!("Could not claim event deliveries: {e}");
                    Vec::new()
                })
            } else {
                Vec::new()
            };
            let full_batch =
                i64::try_from(claimed.len()).unwrap_or(0) == claimable && claimable > 0;

            for delivery in claimed {
                let Ok(permit) = permits.clone().acquire_owned().await else {
                    break;
                };
                let bus = self.clone();
                running.spawn(async move {
                    bus.deliver(delivery).await;
                    drop(permit);
                });
            }

            if last_purge.elapsed() >= PURGE_EVERY {
                last_purge = Instant::now();
                self.purge().await;
            }

            if full_batch {
                continue; // more may be due right now
            }
            tokio::select! {
                () = self.wake.notified() => {}
                () = tokio::time::sleep(self.config.poll_interval) => {}
                Some(_) = running.join_next(), if !running.is_empty() => {}
                _ = stopped.changed() => {}
            }
        }

        // Give running handlers a chance; `ProcessorHandle::stop` bounds this.
        while running.join_next().await.is_some() {}
    }

    /// Runs one delivery and records the outcome.
    async fn deliver(self: Arc<Self>, delivery: Claimed) {
        let Claimed {
            event_seq,
            handler: name,
            attempts,
        } = delivery;
        let attempt = attempts + 1;

        let event = match self
            .blocking(move |bus| store::load_event(&bus.database, event_seq))
            .await
        {
            Ok(event) => event,
            Err(e) => {
                tracing::error!("Could not load event {event_seq}: {e}");
                return; // lease expires, retried later
            }
        };

        let outcome = match self.handler(&event.event_type, &name) {
            None => Err((
                true,
                format!(
                    "no handler named {name} is registered for {}",
                    event.event_type
                ),
            )),
            Some(handler) => {
                match tokio::time::timeout(
                    self.config.handler_timeout,
                    handler(event.clone(), attempt),
                )
                .await
                {
                    Ok(Ok(())) => Ok(()),
                    Ok(Err(error)) => Err((error.permanent, error.message)),
                    Err(_) => Err((
                        false,
                        format!("timed out after {}s", self.config.handler_timeout.as_secs()),
                    )),
                }
            }
        };

        let recorded = match outcome {
            Ok(()) => {
                let handler = name.clone();
                self.blocking(move |bus| store::mark_done(&bus.database, event_seq, &handler))
                    .await
            }
            Err((permanent, message)) => {
                let dead = permanent || attempt >= self.config.max_attempts;
                let status = if dead {
                    DeliveryStatus::Dead
                } else {
                    DeliveryStatus::Failed
                };
                let wait = chrono::Duration::from_std(self.config.backoff_for(attempt))
                    .unwrap_or(chrono::Duration::hours(1));
                if dead {
                    tracing::error!(
                        "Event {} ({event_seq}) is dead for handler {name} after {attempt} attempt(s): {message}",
                        event.event_type
                    );
                } else {
                    tracing::warn!(
                        "Handler {name} failed on {} ({event_seq}), attempt {attempt}: {message}",
                        event.event_type
                    );
                }
                let handler = name.clone();
                self.blocking(move |bus| {
                    store::mark_failed(
                        &bus.database,
                        event_seq,
                        &handler,
                        status,
                        store::now() + wait,
                        &message,
                    )
                })
                .await
            }
        };
        if let Err(e) = recorded {
            tracing::error!("Could not record the outcome of {name} for event {event_seq}: {e}");
        }
    }

    async fn purge(self: &Arc<Self>) {
        let retention =
            chrono::Duration::from_std(self.config.retention).unwrap_or(chrono::Duration::days(30));
        match self
            .blocking(move |bus| store::purge_done(&bus.database, store::now() - retention))
            .await
        {
            Ok(0) => {}
            Ok(n) => tracing::info!("Purged {n} completed event deliveries"),
            Err(e) => tracing::error!("Could not purge event deliveries: {e}"),
        }
    }

    /// Diesel is synchronous; keep it off the async worker threads.
    async fn blocking<T, F>(self: &Arc<Self>, work: F) -> Result<T, EventBusError>
    where
        F: FnOnce(&EventBus) -> Result<T, EventBusError> + Send + 'static,
        T: Send + 'static,
    {
        let bus = self.clone();
        tokio::task::spawn_blocking(move || work(&bus))
            .await
            .unwrap_or_else(|e| {
                Err(EventBusError::Database(
                    diesel::result::Error::QueryBuilderError(
                        format!("event bus task failed: {e}").into(),
                    ),
                ))
            })
    }
}
