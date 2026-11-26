//! SQL access to `events` and `event_deliveries`.
//!
//! Queries are written once in Postgres syntax. For SQLite the `$n`
//! placeholders become `?n` (both reuse a bound value when a number repeats),
//! and the few backend-specific parts – `JSONB` casts and `SKIP LOCKED` –
//! are filled in per backend.

use chrono::{NaiveDateTime, Utc};
use common_infrastructure::db::{DBType, Database, PersistenceError};
use diesel::sql_types::{BigInt, Integer, Nullable, Text, Timestamp};
use diesel::{QueryableByName, RunQueryDsl};
use serde::Serialize;
use std::fmt;

#[derive(Debug, thiserror::Error)]
pub enum EventBusError {
    #[error("event bus database error: {0}")]
    Database(#[from] diesel::result::Error),
    #[error("event bus connection error: {0}")]
    Connection(#[from] PersistenceError),
    #[error("could not serialise event {event_type}: {source}")]
    Serialize {
        event_type: &'static str,
        source: serde_json::Error,
    },
    #[error("no delivery for event {seq} and handler {handler} can be retried")]
    NotRetryable { seq: i64, handler: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DeliveryStatus {
    Pending,
    Processing,
    Done,
    Failed,
    Dead,
}

impl DeliveryStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Processing => "processing",
            Self::Done => "done",
            Self::Failed => "failed",
            Self::Dead => "dead",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "pending" => Self::Pending,
            "processing" => Self::Processing,
            "done" => Self::Done,
            "failed" => Self::Failed,
            "dead" => Self::Dead,
            _ => return None,
        })
    }
}

impl fmt::Display for DeliveryStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// An event as it is written by `publish`.
pub(crate) struct NewEvent<'a> {
    pub id: String,
    pub event_type: &'a str,
    pub version: i32,
    pub aggregate_id: String,
    pub payload: String,
    pub occurred_at: NaiveDateTime,
    pub correlation_id: Option<String>,
}

#[derive(QueryableByName, Debug, Clone)]
pub(crate) struct StoredEvent {
    #[diesel(sql_type = BigInt)]
    pub seq: i64,
    #[diesel(sql_type = Text)]
    pub id: String,
    #[diesel(sql_type = Text)]
    pub event_type: String,
    #[diesel(sql_type = Integer)]
    pub event_version: i32,
    #[diesel(sql_type = Text)]
    pub aggregate_id: String,
    #[diesel(sql_type = Text)]
    pub payload: String,
    #[diesel(sql_type = Timestamp)]
    pub occurred_at: NaiveDateTime,
    #[diesel(sql_type = Nullable<Text>)]
    pub correlation_id: Option<String>,
}

/// A delivery claimed for processing.
#[derive(QueryableByName, Debug, Clone)]
pub(crate) struct Claimed {
    #[diesel(sql_type = BigInt)]
    pub event_seq: i64,
    #[diesel(sql_type = Text)]
    pub handler: String,
    #[diesel(sql_type = Integer)]
    pub attempts: i32,
}

/// A delivery with its event, for the admin view.
#[derive(QueryableByName, Debug, Clone, Serialize)]
pub struct DeliveryView {
    #[diesel(sql_type = BigInt)]
    pub event_seq: i64,
    #[diesel(sql_type = Text)]
    pub event_id: String,
    #[diesel(sql_type = Text)]
    pub event_type: String,
    #[diesel(sql_type = Integer)]
    pub event_version: i32,
    #[diesel(sql_type = Text)]
    pub aggregate_id: String,
    #[diesel(sql_type = Text)]
    pub payload: String,
    #[diesel(sql_type = Timestamp)]
    pub occurred_at: NaiveDateTime,
    #[diesel(sql_type = Text)]
    pub handler: String,
    #[diesel(sql_type = Text)]
    pub status: String,
    #[diesel(sql_type = Integer)]
    pub attempts: i32,
    #[diesel(sql_type = Timestamp)]
    pub next_attempt_at: NaiveDateTime,
    #[diesel(sql_type = Nullable<Text>)]
    pub last_error: Option<String>,
    #[diesel(sql_type = Timestamp)]
    pub updated_at: NaiveDateTime,
}

#[derive(QueryableByName)]
struct Seq {
    #[diesel(sql_type = BigInt)]
    seq: i64,
}

#[derive(QueryableByName)]
struct StatusCount {
    #[diesel(sql_type = Text)]
    status: String,
    #[diesel(sql_type = BigInt)]
    count: i64,
}

fn is_postgres(conn: &DBType) -> bool {
    match conn {
        #[cfg(feature = "postgresql")]
        DBType::Postgresql(_) => true,
        #[cfg(feature = "sqlite")]
        DBType::Sqlite(_) => false,
    }
}

/// Adapts a query written in Postgres syntax to `conn`'s backend.
/// `{jsonb}` marks a value cast to JSONB, `{text}` a JSONB read as text,
/// `{skip_locked}` the row-locking clause of a claim.
fn sql(conn: &DBType, query: &str) -> String {
    if is_postgres(conn) {
        query
            .replace("{jsonb}", "::jsonb")
            .replace("{text}", "::text")
            .replace("{skip_locked}", "FOR UPDATE OF d SKIP LOCKED")
    } else {
        query
            .replace("{jsonb}", "")
            .replace("{text}", "")
            .replace("{skip_locked}", "")
            .replace('$', "?")
    }
}

const EVENT_COLUMNS: &str = "e.seq, e.id, e.event_type, e.event_version, e.aggregate_id, \
     e.payload{text} AS payload, e.occurred_at, e.correlation_id";

pub(crate) fn now() -> NaiveDateTime {
    Utc::now().naive_utc()
}

/// Appends `event` and creates a pending delivery per handler, on `conn`
/// (normally inside the caller's transaction).
pub(crate) fn append(
    conn: &mut DBType,
    event: &NewEvent<'_>,
    handlers: &[&'static str],
) -> Result<i64, EventBusError> {
    let query = sql(
        conn,
        "INSERT INTO events (id, event_type, event_version, aggregate_id, payload, occurred_at, correlation_id) \
         VALUES ($1, $2, $3, $4, $5{jsonb}, $6, $7) RETURNING seq",
    );
    let Seq { seq } = diesel::sql_query(query)
        .bind::<Text, _>(&event.id)
        .bind::<Text, _>(event.event_type)
        .bind::<Integer, _>(event.version)
        .bind::<Text, _>(&event.aggregate_id)
        .bind::<Text, _>(&event.payload)
        .bind::<Timestamp, _>(event.occurred_at)
        .bind::<Nullable<Text>, _>(event.correlation_id.as_deref())
        .get_result(conn)?;

    let insert_delivery = sql(
        conn,
        "INSERT INTO event_deliveries (event_seq, handler, status, attempts, next_attempt_at, updated_at) \
         VALUES ($1, $2, 'pending', 0, $3, $3)",
    );
    for handler in handlers {
        diesel::sql_query(&insert_delivery)
            .bind::<BigInt, _>(seq)
            .bind::<Text, _>(*handler)
            .bind::<Timestamp, _>(event.occurred_at)
            .execute(conn)?;
    }
    Ok(seq)
}

/// Leases up to `limit` due deliveries until `lease_until`.
///
/// Due means pending/failed with `next_attempt_at` reached, or processing
/// with an expired lease (the worker died). A delivery is skipped while the
/// same handler still has an earlier, unfinished delivery for the same
/// aggregate, which keeps each handler's view of an aggregate in order.
/// `dead` deliveries do not block later ones.
pub(crate) fn claim(
    database: &Database,
    limit: i64,
    lease_until: NaiveDateTime,
) -> Result<Vec<Claimed>, EventBusError> {
    let mut conn = database.connection()?;
    let query = sql(
        &conn,
        "UPDATE event_deliveries SET status = 'processing', locked_until = $2, updated_at = $1 \
         WHERE (event_seq, handler) IN ( \
             SELECT d.event_seq, d.handler FROM event_deliveries d \
             JOIN events e ON e.seq = d.event_seq \
             WHERE ((d.status IN ('pending', 'failed') AND d.next_attempt_at <= $1) \
                    OR (d.status = 'processing' AND d.locked_until < $1)) \
               AND NOT EXISTS ( \
                   SELECT 1 FROM event_deliveries d2 JOIN events e2 ON e2.seq = d2.event_seq \
                   WHERE d2.handler = d.handler AND e2.aggregate_id = e.aggregate_id \
                     AND d2.event_seq < d.event_seq AND d2.status NOT IN ('done', 'dead')) \
             ORDER BY d.event_seq LIMIT $3 {skip_locked}) \
         RETURNING event_seq, handler, attempts",
    );
    let mut claimed: Vec<Claimed> = diesel::sql_query(query)
        .bind::<Timestamp, _>(now())
        .bind::<Timestamp, _>(lease_until)
        .bind::<BigInt, _>(limit)
        .load(&mut *conn)?;
    claimed.sort_by_key(|c| c.event_seq);
    Ok(claimed)
}

pub(crate) fn load_event(database: &Database, seq: i64) -> Result<StoredEvent, EventBusError> {
    let mut conn = database.connection()?;
    let query = sql(
        &conn,
        &format!("SELECT {EVENT_COLUMNS} FROM events e WHERE e.seq = $1"),
    );
    Ok(diesel::sql_query(query)
        .bind::<BigInt, _>(seq)
        .get_result(&mut *conn)?)
}

pub(crate) fn mark_done(database: &Database, seq: i64, handler: &str) -> Result<(), EventBusError> {
    let mut conn = database.connection()?;
    let query = sql(
        &conn,
        "UPDATE event_deliveries SET status = 'done', attempts = attempts + 1, locked_until = NULL, \
         last_error = NULL, updated_at = $3 WHERE event_seq = $1 AND handler = $2",
    );
    diesel::sql_query(query)
        .bind::<BigInt, _>(seq)
        .bind::<Text, _>(handler)
        .bind::<Timestamp, _>(now())
        .execute(&mut *conn)?;
    Ok(())
}

/// Records a failed attempt; `next_attempt_at` is ignored for `dead`.
pub(crate) fn mark_failed(
    database: &Database,
    seq: i64,
    handler: &str,
    status: DeliveryStatus,
    next_attempt_at: NaiveDateTime,
    error: &str,
) -> Result<(), EventBusError> {
    let mut conn = database.connection()?;
    let query = sql(
        &conn,
        "UPDATE event_deliveries SET status = $3, attempts = attempts + 1, next_attempt_at = $4, \
         last_error = $5, locked_until = NULL, updated_at = $6 WHERE event_seq = $1 AND handler = $2",
    );
    diesel::sql_query(query)
        .bind::<BigInt, _>(seq)
        .bind::<Text, _>(handler)
        .bind::<Text, _>(status.as_str())
        .bind::<Timestamp, _>(next_attempt_at)
        .bind::<Text, _>(error)
        .bind::<Timestamp, _>(now())
        .execute(&mut *conn)?;
    Ok(())
}

pub(crate) fn deliveries(
    database: &Database,
    status: Option<DeliveryStatus>,
    limit: i64,
) -> Result<Vec<DeliveryView>, EventBusError> {
    let mut conn = database.connection()?;
    let query = sql(
        &conn,
        "SELECT d.event_seq, e.id AS event_id, e.event_type, e.event_version, e.aggregate_id, \
         e.payload{text} AS payload, e.occurred_at, d.handler, d.status, d.attempts, \
         d.next_attempt_at, d.last_error, d.updated_at \
         FROM event_deliveries d JOIN events e ON e.seq = d.event_seq \
         WHERE ($1 IS NULL OR d.status = $1) ORDER BY d.event_seq DESC LIMIT $2",
    );
    Ok(diesel::sql_query(query)
        .bind::<Nullable<Text>, _>(status.map(DeliveryStatus::as_str))
        .bind::<BigInt, _>(limit)
        .load(&mut *conn)?)
}

pub(crate) fn counts(database: &Database) -> Result<Vec<(String, i64)>, EventBusError> {
    let mut conn = database.connection()?;
    let rows: Vec<StatusCount> =
        diesel::sql_query("SELECT status, COUNT(*) AS count FROM event_deliveries GROUP BY status")
            .load(&mut *conn)?;
    Ok(rows.into_iter().map(|r| (r.status, r.count)).collect())
}

/// Puts a failed or dead delivery back in the queue, with a fresh attempt count.
pub(crate) fn retry(database: &Database, seq: i64, handler: &str) -> Result<(), EventBusError> {
    let mut conn = database.connection()?;
    let query = sql(
        &conn,
        "UPDATE event_deliveries SET status = 'pending', attempts = 0, next_attempt_at = $3, \
         last_error = NULL, locked_until = NULL, updated_at = $3 \
         WHERE event_seq = $1 AND handler = $2 AND status IN ('failed', 'dead')",
    );
    let updated = diesel::sql_query(query)
        .bind::<BigInt, _>(seq)
        .bind::<Text, _>(handler)
        .bind::<Timestamp, _>(now())
        .execute(&mut *conn)?;
    if updated == 0 {
        return Err(EventBusError::NotRetryable {
            seq,
            handler: handler.to_owned(),
        });
    }
    Ok(())
}

/// Deletes completed deliveries older than `before`. Events stay as an audit log.
pub(crate) fn purge_done(
    database: &Database,
    before: NaiveDateTime,
) -> Result<usize, EventBusError> {
    let mut conn = database.connection()?;
    let query = sql(
        &conn,
        "DELETE FROM event_deliveries WHERE status = 'done' AND updated_at < $1",
    );
    Ok(diesel::sql_query(query)
        .bind::<Timestamp, _>(before)
        .execute(&mut *conn)?)
}
