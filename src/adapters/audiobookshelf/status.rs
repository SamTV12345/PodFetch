use actix_web::{get, HttpResponse};
use crate::adapters::audiobookshelf::models::status::StatusResponse;

#[get("/status")]
pub async fn get_status() -> HttpResponse {
    HttpResponse::Ok().json(StatusResponse::new())
}