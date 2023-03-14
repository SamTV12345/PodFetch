
use actix_web::{get,put, HttpResponse, Responder, web};
use crate::db::DB;

#[get("/notifications/unread")]
pub async fn get_unread_notifications() -> impl Responder {
    let mut db = DB::new().unwrap();
    let notifications = db.get_unread_notifications();
    HttpResponse::Ok().json(notifications.unwrap())
}

#[derive(Deserialize)]
pub struct NotificationId {
    id: i32
}

#[put("/notifications/dismiss")]
pub async fn dismiss_notifications(id: web::Json<NotificationId>) -> impl Responder {
    let mut db = DB::new().unwrap();
     db.update_status_of_notification(id.id, "dismissed")
         .expect("Error dismissing notification");
    HttpResponse::Ok()
}