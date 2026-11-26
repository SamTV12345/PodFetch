//! Tests against a real database: SQLite (a fresh file per test) or, with
//! only the `postgresql` feature, a fresh Postgres database per test.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use common_infrastructure::db::Database;
use serde::{Deserialize, Serialize};

use crate::{DeliveryStatus, Event, EventBus, EventBusConfig, EventBusError, HandlerError};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct EpisodeDiscovered {
    podcast: String,
    episode: u32,
}

impl Event for EpisodeDiscovered {
    const TYPE: &'static str = "test.episode_discovered";
    fn aggregate_id(&self) -> String {
        self.podcast.clone()
    }
}

/// Same type name, newer schema: `title` was added in version 2.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EpisodeDiscoveredV2 {
    podcast: String,
    episode: u32,
    title: String,
}

impl Event for EpisodeDiscoveredV2 {
    const TYPE: &'static str = "test.episode_discovered";
    const VERSION: i32 = 2;
    fn aggregate_id(&self) -> String {
        self.podcast.clone()
    }
    fn upcast(version: i32, mut payload: serde_json::Value) -> Result<serde_json::Value, String> {
        match version {
            1 => {
                payload["title"] = serde_json::Value::from("untitled");
                Ok(payload)
            }
            2 => Ok(payload),
            other => Err(format!("unknown version {other}")),
        }
    }
}

fn discovered(podcast: &str, episode: u32) -> EpisodeDiscovered {
    EpisodeDiscovered {
        podcast: podcast.to_owned(),
        episode,
    }
}

fn config() -> EventBusConfig {
    EventBusConfig {
        poll_interval: Duration::from_millis(50),
        handler_timeout: Duration::from_secs(2),
        max_attempts: 3,
        backoff: vec![Duration::from_millis(50)],
        ..EventBusConfig::default()
    }
}

#[cfg(feature = "sqlite")]
async fn test_database() -> Database {
    let path = std::env::temp_dir().join(format!("pf-events-{}.db", uuid::Uuid::new_v4()));
    let database = Database::from_url(&format!("sqlite://{}", path.display()), 4)
        .expect("sqlite test database");
    podfetch_persistence::migrate(&database);
    database
}

#[cfg(all(feature = "postgresql", not(feature = "sqlite")))]
async fn test_database() -> Database {
    use diesel::RunQueryDsl;

    const SERVER: &str = "postgres://postgres:postgres@127.0.0.1:55002";
    ensure_postgres().await;
    let name = format!("pf_events_{}", uuid::Uuid::new_v4().simple());
    let admin = Database::from_url(&format!("{SERVER}/postgres"), 2).expect("postgres admin");
    diesel::sql_query(format!("CREATE DATABASE {name}"))
        .execute(&mut *admin.connection().expect("admin connection"))
        .expect("create test database");
    let database = Database::from_url(&format!("{SERVER}/{name}"), 4).expect("postgres test db");
    podfetch_persistence::migrate(&database);
    database
}

/// Uses a Postgres already listening on 55002 (CI's shared container or a
/// local server), otherwise starts one.
#[cfg(all(feature = "postgresql", not(feature = "sqlite")))]
async fn ensure_postgres() {
    use testcontainers::ImageExt;
    use testcontainers::runners::AsyncRunner;
    use testcontainers_modules::postgres::Postgres;
    static STARTED: tokio::sync::OnceCell<()> = tokio::sync::OnceCell::const_new();

    if tokio::net::TcpStream::connect("127.0.0.1:55002")
        .await
        .is_ok()
    {
        return;
    }
    STARTED
        .get_or_init(|| async {
            let container = Postgres::default()
                .with_tag("18-alpine")
                .with_mapped_port(55002, 5432.into())
                .start()
                .await
                .expect("postgres test container");
            std::mem::forget(container);
        })
        .await;
}

async fn bus() -> Arc<EventBus> {
    EventBus::new(test_database().await, config())
}

/// Polls `check` until it holds or 10 s have passed.
async fn eventually(what: &str, check: impl Fn() -> bool) {
    for _ in 0..200 {
        if check() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("timed out waiting for: {what}");
}

fn statuses(bus: &EventBus, handler: &str) -> Vec<String> {
    let mut deliveries = bus.deliveries(None, 100).expect("deliveries");
    deliveries.retain(|d| d.handler == handler);
    deliveries.sort_by_key(|d| d.event_seq);
    deliveries.into_iter().map(|d| d.status).collect()
}

type Seen = Arc<Mutex<Vec<EpisodeDiscovered>>>;

fn recorder(bus: &EventBus, name: &'static str) -> Seen {
    let seen: Seen = Arc::default();
    let sink = seen.clone();
    bus.subscribe(name, move |event: EpisodeDiscovered, _| {
        let sink = sink.clone();
        async move {
            sink.lock().unwrap().push(event);
            Ok(())
        }
    });
    seen
}

#[tokio::test(flavor = "multi_thread")]
async fn publish_creates_one_delivery_per_handler_and_rollback_keeps_nothing() {
    let bus = bus().await;
    let _a = recorder(&bus, "test.a");
    let _b = recorder(&bus, "test.b");

    bus.publish(&discovered("p1", 1)).unwrap();
    let rolled_back: Result<(), EventBusError> = bus.transaction(|tx| {
        tx.publish(&discovered("p1", 2))?;
        Err(EventBusError::NotRetryable {
            seq: 0,
            handler: "abort".to_owned(),
        })
    });
    assert!(rolled_back.is_err());

    let deliveries = bus.deliveries(None, 100).unwrap();
    assert_eq!(
        deliveries.len(),
        2,
        "only the committed event has deliveries"
    );
    assert!(deliveries.iter().all(|d| d.status == "pending"));
    let payload: serde_json::Value = serde_json::from_str(&deliveries[0].payload).unwrap();
    assert_eq!(payload["episode"], 1);
}

#[tokio::test(flavor = "multi_thread")]
async fn processor_delivers_typed_events_to_every_handler() {
    let bus = bus().await;
    let a = recorder(&bus, "test.a");
    let b = recorder(&bus, "test.b");
    let processor = bus.spawn_processor();

    bus.publish(&discovered("p1", 7)).unwrap();

    eventually("both handlers ran", || {
        a.lock().unwrap().len() == 1 && b.lock().unwrap().len() == 1
    })
    .await;
    assert_eq!(a.lock().unwrap()[0], discovered("p1", 7));
    eventually("deliveries are done", || {
        statuses(&bus, "test.a") == ["done"] && statuses(&bus, "test.b") == ["done"]
    })
    .await;
    processor.stop(Duration::from_secs(2)).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_failing_handler_retries_until_dead_without_affecting_others() {
    let bus = bus().await;
    let ok = recorder(&bus, "test.ok");
    let attempts = Arc::new(Mutex::new(Vec::new()));
    let seen = attempts.clone();
    bus.subscribe("test.flaky", move |_: EpisodeDiscovered, meta| {
        let seen = seen.clone();
        async move {
            seen.lock().unwrap().push(meta.attempt);
            Err::<(), _>(HandlerError::retry("feed server down"))
        }
    });
    let processor = bus.spawn_processor();

    bus.publish(&discovered("p1", 1)).unwrap();

    eventually("flaky handler is dead", || {
        statuses(&bus, "test.flaky") == ["dead"]
    })
    .await;
    assert_eq!(*attempts.lock().unwrap(), [1, 2, 3], "max_attempts = 3");
    assert_eq!(ok.lock().unwrap().len(), 1, "the other handler ran once");
    assert_eq!(statuses(&bus, "test.ok"), ["done"]);

    let dead = bus.deliveries(Some(DeliveryStatus::Dead), 10).unwrap();
    assert_eq!(dead[0].attempts, 3);
    assert_eq!(dead[0].last_error.as_deref(), Some("feed server down"));
    processor.stop(Duration::from_secs(2)).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn permanent_errors_skip_retries_and_a_retry_requeues() {
    let bus = bus().await;
    let fixed = Arc::new(Mutex::new(false));
    let calls = Arc::new(Mutex::new(0));
    let (is_fixed, count) = (fixed.clone(), calls.clone());
    bus.subscribe("test.strict", move |_: EpisodeDiscovered, _| {
        let (is_fixed, count) = (is_fixed.clone(), count.clone());
        async move {
            *count.lock().unwrap() += 1;
            if *is_fixed.lock().unwrap() {
                Ok(())
            } else {
                Err(HandlerError::permanent("bad data"))
            }
        }
    });
    let processor = bus.spawn_processor();

    bus.publish(&discovered("p1", 1)).unwrap();
    eventually("dead after one attempt", || {
        statuses(&bus, "test.strict") == ["dead"]
    })
    .await;
    assert_eq!(*calls.lock().unwrap(), 1);

    *fixed.lock().unwrap() = true;
    let dead = &bus.deliveries(Some(DeliveryStatus::Dead), 10).unwrap()[0];
    bus.retry(dead.event_seq, &dead.handler).unwrap();
    eventually("done after the retry", || {
        statuses(&bus, "test.strict") == ["done"]
    })
    .await;
    assert!(
        bus.retry(dead.event_seq, &dead.handler).is_err(),
        "a done delivery cannot be retried"
    );
    processor.stop(Duration::from_secs(2)).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn deliveries_of_one_aggregate_stay_in_order() {
    let bus = bus().await;
    let order = Arc::new(Mutex::new(Vec::new()));
    let failed_once = Arc::new(Mutex::new(false));
    let (sink, flag) = (order.clone(), failed_once.clone());
    bus.subscribe("test.ordered", move |event: EpisodeDiscovered, _| {
        let (sink, flag) = (sink.clone(), flag.clone());
        async move {
            // The first episode of p1 fails once; its successors must wait.
            if event.podcast == "p1" && event.episode == 1 && !*flag.lock().unwrap() {
                *flag.lock().unwrap() = true;
                return Err(HandlerError::retry("first try fails"));
            }
            sink.lock().unwrap().push((event.podcast, event.episode));
            Ok(())
        }
    });

    for episode in 1..=3 {
        bus.publish(&discovered("p1", episode)).unwrap();
    }
    bus.publish(&discovered("p2", 1)).unwrap();
    let processor = bus.spawn_processor();

    eventually("all four delivered", || order.lock().unwrap().len() == 4).await;
    let order = order.lock().unwrap().clone();
    let p1: Vec<u32> = order
        .iter()
        .filter(|(p, _)| p == "p1")
        .map(|(_, e)| *e)
        .collect();
    assert_eq!(p1, [1, 2, 3], "p1 stays in order despite the retry");
    processor.stop(Duration::from_secs(2)).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn old_payloads_are_upcast_and_unreadable_ones_are_dead() {
    let bus = bus().await;
    let titles = Arc::new(Mutex::new(Vec::new()));
    let sink = titles.clone();
    bus.subscribe("test.v2", move |event: EpisodeDiscoveredV2, _| {
        let sink = sink.clone();
        async move {
            sink.lock().unwrap().push(event.title);
            Ok(())
        }
    });

    // Stored as version 1 (the old struct), read by the version 2 handler.
    bus.publish(&discovered("p1", 1)).unwrap();
    bus.publish(&EpisodeDiscoveredV2 {
        podcast: "p1".to_owned(),
        episode: 2,
        title: "Pilot".to_owned(),
    })
    .unwrap();
    let processor = bus.spawn_processor();

    eventually("both delivered", || titles.lock().unwrap().len() == 2).await;
    assert_eq!(*titles.lock().unwrap(), ["untitled", "Pilot"]);
    processor.stop(Duration::from_secs(2)).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn purge_removes_only_old_completed_deliveries() {
    let bus = bus().await;
    let _a = recorder(&bus, "test.a");
    let processor = bus.spawn_processor();
    bus.publish(&discovered("p1", 1)).unwrap();
    eventually("done", || statuses(&bus, "test.a") == ["done"]).await;
    processor.stop(Duration::from_secs(2)).await;

    let past = crate::store::now() - chrono::Duration::days(1);
    assert_eq!(crate::store::purge_done(&bus.database, past).unwrap(), 0);
    let future = crate::store::now() + chrono::Duration::days(1);
    assert_eq!(crate::store::purge_done(&bus.database, future).unwrap(), 1);
    assert_eq!(bus.counts().unwrap(), Vec::new());
}

#[test]
fn backoff_repeats_its_last_step() {
    let config = EventBusConfig::default();
    assert_eq!(config.backoff_for(1), Duration::from_secs(5));
    assert_eq!(config.backoff_for(5), Duration::from_secs(3600));
    assert_eq!(config.backoff_for(9), Duration::from_secs(3600));
}
