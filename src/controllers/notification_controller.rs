use crate::DbPool;
use actix_web::web::Data;
use actix_web::{get, put, web, HttpResponse};
use std::ops::DerefMut;
use std::sync::Mutex;

use crate::mutex::LockResultExt;
use crate::service::notification_service::NotificationService;
use crate::utils::error::{map_r2d2_error, CustomError};

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets all unread notifications.",body= Vec<Notification>)),
tag="notifications"
)]
#[get("/notifications/unread")]
pub async fn get_unread_notifications(
    notification_service: Data<Mutex<NotificationService>>,
    conn: Data<DbPool>,
) -> Result<HttpResponse, CustomError> {
    let notifications = notification_service
        .lock()
        .ignore_poison()
        .get_unread_notifications(conn.get().map_err(map_r2d2_error)?.deref_mut())?;
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
    notification_service: Data<Mutex<NotificationService>>,
    conn: Data<DbPool>,
) -> Result<HttpResponse, CustomError> {
    notification_service
        .lock()
        .ignore_poison()
        .update_status_of_notification(
            id.id,
            "dismissed",
            conn.get().map_err(map_r2d2_error)?.deref_mut(),
        )?;
    Ok(HttpResponse::Ok().body(""))
}
