use crate::models::notification::Notification;
use crate::service::notification_service::NotificationService;
use crate::utils::error::CustomError;
use actix_web::{get, put, web, HttpResponse};
use utoipa::ToSchema;
#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets all unread notifications.",body= Vec<Notification>)),
tag="notifications"
)]
#[get("/notifications/unread")]
pub async fn get_unread_notifications() -> Result<HttpResponse, CustomError> {
    let notifications = NotificationService::get_unread_notifications()?;
    Ok(HttpResponse::Ok().json(notifications))
}

#[derive(Deserialize, ToSchema)]
pub struct NotificationId {
    id: i32,
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Dismisses a notification")),
tag="notifications"
)]
#[put("/notifications/dismiss")]
pub async fn dismiss_notifications(
    id: web::Json<NotificationId>,
) -> Result<HttpResponse, CustomError> {
    NotificationService::update_status_of_notification(id.id, "dismissed")?;
    Ok(HttpResponse::Ok().body(""))
}
