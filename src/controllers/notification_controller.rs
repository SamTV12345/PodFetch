use axum::Json;
use axum::routing::{get, put};
use reqwest::StatusCode;
use crate::models::notification::Notification;
use crate::service::notification_service::NotificationService;
use crate::utils::error::CustomError;

use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;

#[utoipa::path(
get,
path="/notifications/unread",
responses(
(status = 200, description = "Gets all unread notifications.",body= Vec<Notification>)),
tag="notifications"
)]
pub async fn get_unread_notifications() -> Result<Json<Vec<Notification>>, CustomError> {
    let notifications = NotificationService::get_unread_notifications()?;
    Ok(Json(notifications))
}

#[derive(Deserialize, ToSchema)]
pub struct NotificationId {
    id: i32,
}

#[utoipa::path(
put,
path="/notifications/dismiss",
responses(
(status = 200, description = "Dismisses a notification")),
tag="notifications"
)]
pub async fn dismiss_notifications(
    Json(id): Json<NotificationId>,
) -> Result<StatusCode, CustomError> {
    NotificationService::update_status_of_notification(id.id, "dismissed")?;
    Ok(StatusCode::OK)
}

pub fn get_notification_router() -> OpenApiRouter {
    OpenApiRouter::new()
        .route("/notifications/unread", get(get_unread_notifications))
        .route("/notifications/dismiss", put(dismiss_notifications))
}