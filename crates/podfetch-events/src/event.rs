use chrono::{DateTime, Utc};
use serde::Serialize;
use serde::de::DeserializeOwned;
use uuid::Uuid;

/// A domain event that can be stored and delivered by the [`crate::EventBus`].
pub trait Event: Serialize + DeserializeOwned + Send + Sync + 'static {
    /// Stable, globally unique name, `<context>.<what_happened>`, e.g.
    /// `catalog.episode_discovered`. Never change it once events are stored.
    const TYPE: &'static str;

    /// Schema version of the payload. Bump it when the payload changes
    /// incompatibly and teach [`Event::upcast`] to read the old shape.
    const VERSION: i32 = 1;

    /// The aggregate this event belongs to. Deliveries of one handler are
    /// processed in order per aggregate id.
    fn aggregate_id(&self) -> String;

    /// Converts a payload stored with `version` into the current shape.
    /// The default accepts only the current version.
    fn upcast(version: i32, payload: serde_json::Value) -> Result<serde_json::Value, String> {
        if version == Self::VERSION {
            Ok(payload)
        } else {
            Err(format!(
                "{} has no upcast from version {version} to {}",
                Self::TYPE,
                Self::VERSION
            ))
        }
    }
}

/// Envelope data a handler gets alongside the event.
#[derive(Debug, Clone)]
pub struct EventMeta {
    pub id: Uuid,
    pub seq: i64,
    pub aggregate_id: String,
    pub occurred_at: DateTime<Utc>,
    pub correlation_id: Option<Uuid>,
    /// 1 on the first delivery, 2 on the first retry, …
    pub attempt: i32,
}

/// Why a handler failed. `?` converts any error into a retryable failure;
/// use [`HandlerError::permanent`] for failures a retry cannot fix.
#[derive(Debug)]
pub struct HandlerError {
    pub(crate) message: String,
    pub(crate) permanent: bool,
}

impl HandlerError {
    /// A failure worth retrying later (network down, file locked, …).
    pub fn retry(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            permanent: false,
        }
    }

    /// A failure no retry can fix; the delivery goes straight to `dead`.
    pub fn permanent(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            permanent: true,
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl<E: std::error::Error> From<E> for HandlerError {
    fn from(error: E) -> Self {
        Self::retry(error.to_string())
    }
}
