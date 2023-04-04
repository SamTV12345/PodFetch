use actix_web::web::Data;
use actix_web::{get, put, web, HttpResponse, Responder};
use std::sync::Mutex;
use crate::service::notification_service::NotificationService;

#[get("/notifications/unread")]
pub async fn get_unread_notifications(db: Data<Mutex<NotificationService>>) -> impl Responder {
    let notifications = db
        .lock()
        .expect("Error locking db")
        .get_unread_notifications();
    HttpResponse::Ok().json(notifications.unwrap())
}

#[derive(Deserialize)]
pub struct NotificationId {
    id: i32,
}

#[put("/notifications/dismiss")]
pub async fn dismiss_notifications(
    id: web::Json<NotificationId>,
    notification_service: Data<Mutex<NotificationService>>,
) -> impl Responder {
    notification_service.lock()
        .expect("Error locking db")
        .update_status_of_notification(id.id, "dismissed")
        .expect("Error dismissing notification");
    HttpResponse::Ok()
}
