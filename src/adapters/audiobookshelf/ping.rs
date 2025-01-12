use actix_web::{get, HttpResponse};

#[derive(Serialize)]
pub struct PingResponse {
    success: bool,
}

impl PingResponse {
    pub fn new() -> Self {
        Self { success: true }
    }
}

#[get("/ping")]
pub async fn ping() -> HttpResponse {
    HttpResponse::Ok().json(PingResponse::new())
}