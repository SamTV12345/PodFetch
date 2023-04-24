use actix_web::{HttpResponse, Responder};

pub mod episodes;
use actix_web::get;

#[derive(Serialize, Deserialize)]
pub struct EpisodeActionResponse{
    actions: Vec<String>,
    timestamp: i64
}

#[get("/episodes/{username}.json")]
pub async fn get_episode_actions() -> impl Responder {
    HttpResponse::Ok().json(EpisodeActionResponse{
        actions: vec![],
        timestamp: 0
    })
}