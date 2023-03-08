
use actix_web::{get, HttpRequest, HttpResponse, Responder};
use crate::db::DB;

#[get("/notifications/unread")]
pub async fn get_unread_notifications() -> impl Responder {
    let mut db = DB::new().unwrap();
    let notifications = db.get_unread_notifications();
    HttpResponse::Ok().json(notifications)
}