use actix_web::web::Data;
use actix_web::{get, put, web, HttpResponse, Responder};
use std::sync::Mutex;
use crate::{DbPool};
use crate::models::notification::Notification;
use crate::mutex::LockResultExt;
use crate::service::notification_service::NotificationService;
use crate::utils::error::CustomError;

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets all unread notifications.",body= Vec<Notification>)),
tag="notifications"
)]
#[get("/notifications/unread")]
pub async fn get_unread_notifications(notification_service: Data<Mutex<NotificationService>>, conn: Data<DbPool> ) ->
                                                                                             Result<HttpResponse, CustomError> {
    let notifications = notification_service
        .lock().ignore_poison()
        .get_unread_notifications(&mut conn.get().unwrap())?;
    Ok(HttpResponse::Ok().json(notifications))
}

#[derive(Deserialize)]
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
    notification_service: Data<Mutex<NotificationService>>, conn: Data<DbPool>
) -> Result<HttpResponse,CustomError> {
    notification_service.lock()
        .ignore_poison()
        .update_status_of_notification(id.id, "dismissed",
                                       &mut conn.get().unwrap())?;
    Ok(HttpResponse::Ok().body(""))
}
