//! Admin view of the event bus: what is waiting, what failed, and a way to
//! re-run dead deliveries.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use common_infrastructure::error::{CustomError, CustomErrorInner, ErrorSeverity};
use podfetch_events::{DeliveryStatus, DeliveryView, EventBusError};
use podfetch_identity::domain::user::User;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::app_state::AppState;

const DEFAULT_LIMIT: i64 = 100;
const MAX_LIMIT: i64 = 1000;

#[derive(Serialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EventDeliveryDto {
    pub event_seq: i64,
    pub event_id: String,
    pub event_type: String,
    pub event_version: i32,
    pub aggregate_id: String,
    #[schema(value_type = Object)]
    pub payload: serde_json::Value,
    pub occurred_at: String,
    pub handler: String,
    pub status: String,
    pub attempts: i32,
    pub next_attempt_at: String,
    pub last_error: Option<String>,
    pub updated_at: String,
}

impl From<DeliveryView> for EventDeliveryDto {
    fn from(view: DeliveryView) -> Self {
        Self {
            event_seq: view.event_seq,
            payload: serde_json::from_str(&view.payload)
                .unwrap_or(serde_json::Value::String(view.payload)),
            event_id: view.event_id,
            event_type: view.event_type,
            event_version: view.event_version,
            aggregate_id: view.aggregate_id,
            occurred_at: view.occurred_at.and_utc().to_rfc3339(),
            handler: view.handler,
            status: view.status,
            attempts: view.attempts,
            next_attempt_at: view.next_attempt_at.and_utc().to_rfc3339(),
            last_error: view.last_error,
            updated_at: view.updated_at.and_utc().to_rfc3339(),
        }
    }
}

#[derive(Serialize, Debug, Clone, Default, ToSchema)]
pub struct EventStatsDto {
    pub pending: i64,
    pub processing: i64,
    pub done: i64,
    pub failed: i64,
    pub dead: i64,
}

#[derive(Deserialize, Debug, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct DeliveryQuery {
    /// pending, processing, done, failed or dead; all when omitted
    pub status: Option<String>,
    /// at most 1000, default 100
    pub limit: Option<i64>,
}

fn require_admin(user: &User) -> Result<(), CustomError> {
    if user.is_admin() {
        Ok(())
    } else {
        Err(CustomErrorInner::Forbidden(ErrorSeverity::Warning).into())
    }
}

fn map_event_error(error: EventBusError) -> CustomError {
    match error {
        EventBusError::NotRetryable { .. } => {
            CustomErrorInner::Conflict(error.to_string(), ErrorSeverity::Info).into()
        }
        other => {
            tracing::error!("Event bus error: {other}");
            CustomErrorInner::Unknown(ErrorSeverity::Error).into()
        }
    }
}

#[utoipa::path(
    get,
    path = "/events/deliveries",
    params(DeliveryQuery),
    responses(
        (status = 200, description = "Event deliveries, newest first", body = Vec<EventDeliveryDto>),
        (status = 400, description = "Unknown status"),
        (status = 403, description = "Admin only")
    ),
    tag = "events"
)]
pub async fn list_event_deliveries(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Query(query): Query<DeliveryQuery>,
) -> Result<Json<Vec<EventDeliveryDto>>, CustomError> {
    require_admin(&user)?;
    let status = query
        .status
        .as_deref()
        .map(|s| {
            DeliveryStatus::parse(s).ok_or_else(|| {
                CustomError::from(CustomErrorInner::BadRequest(
                    format!("Unknown delivery status '{s}'"),
                    ErrorSeverity::Info,
                ))
            })
        })
        .transpose()?;
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let deliveries = state
        .event_bus
        .deliveries(status, limit)
        .map_err(map_event_error)?;
    Ok(Json(deliveries.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    get,
    path = "/events/stats",
    responses(
        (status = 200, description = "Number of deliveries per status", body = EventStatsDto),
        (status = 403, description = "Admin only")
    ),
    tag = "events"
)]
pub async fn get_event_stats(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<EventStatsDto>, CustomError> {
    require_admin(&user)?;
    let mut stats = EventStatsDto::default();
    for (status, count) in state.event_bus.counts().map_err(map_event_error)? {
        match DeliveryStatus::parse(&status) {
            Some(DeliveryStatus::Pending) => stats.pending = count,
            Some(DeliveryStatus::Processing) => stats.processing = count,
            Some(DeliveryStatus::Done) => stats.done = count,
            Some(DeliveryStatus::Failed) => stats.failed = count,
            Some(DeliveryStatus::Dead) => stats.dead = count,
            None => tracing::warn!("Unknown delivery status '{status}' in the database"),
        }
    }
    Ok(Json(stats))
}

#[utoipa::path(
    post,
    path = "/events/deliveries/{event_seq}/{handler}/retry",
    params(
        ("event_seq" = i64, Path, description = "Sequence number of the event"),
        ("handler" = String, Path, description = "Name of the handler")
    ),
    responses(
        (status = 204, description = "Delivery re-queued"),
        (status = 403, description = "Admin only"),
        (status = 409, description = "Delivery is not failed or dead")
    ),
    tag = "events"
)]
pub async fn retry_event_delivery(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
    Path((event_seq, handler)): Path<(i64, String)>,
) -> Result<StatusCode, CustomError> {
    require_admin(&user)?;
    state
        .event_bus
        .retry(event_seq, &handler)
        .map_err(map_event_error)?;
    Ok(StatusCode::NO_CONTENT)
}

pub fn get_event_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list_event_deliveries))
        .routes(routes!(get_event_stats))
        .routes(routes!(retry_event_delivery))
}

#[cfg(test)]
mod tests {
    use crate::test_support::tests::{app_state, handle_test_startup};
    use podfetch_events::{Event, HandlerError};
    use serde::{Deserialize, Serialize};
    use serde_json::Value;
    use std::time::Duration;

    #[derive(Serialize, Deserialize)]
    struct Pinged {
        id: u32,
    }

    impl Event for Pinged {
        const TYPE: &'static str = "test.pinged";
        fn aggregate_id(&self) -> String {
            self.id.to_string()
        }
    }

    #[tokio::test]
    async fn admin_sees_dead_deliveries_and_can_retry_them() {
        let server = handle_test_startup().await;
        let state = app_state();
        state
            .event_bus
            .subscribe("test.always_fails", |_: Pinged, _| async {
                Err::<(), _>(HandlerError::permanent("broken on purpose"))
            });
        let processor = state.event_bus.spawn_processor();
        state.event_bus.publish(&Pinged { id: 42 }).unwrap();

        let mut dead = Vec::new();
        for _ in 0..100 {
            let response = server
                .test_server
                .get("/api/v1/events/deliveries")
                .add_query_param("status", "dead")
                .await;
            response.assert_status_ok();
            dead = response.json::<Vec<Value>>();
            if !dead.is_empty() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        processor.stop(Duration::from_secs(1)).await;

        assert_eq!(dead.len(), 1, "the failing delivery is dead");
        assert_eq!(dead[0]["eventType"], "test.pinged");
        assert_eq!(dead[0]["payload"]["id"], 42);
        assert_eq!(dead[0]["lastError"], "broken on purpose");

        let stats = server.test_server.get("/api/v1/events/stats").await;
        stats.assert_status_ok();
        assert_eq!(stats.json::<Value>()["dead"], 1);

        let seq = dead[0]["eventSeq"].as_i64().unwrap();
        let retry = format!("/api/v1/events/deliveries/{seq}/test.always_fails/retry");
        server
            .test_server
            .post(&retry)
            .await
            .assert_status(axum::http::StatusCode::NO_CONTENT);
        server
            .test_server
            .post(&retry)
            .await
            .assert_status(axum::http::StatusCode::CONFLICT);

        server
            .test_server
            .get("/api/v1/events/deliveries")
            .add_query_param("status", "nonsense")
            .await
            .assert_status_bad_request();
    }
}
