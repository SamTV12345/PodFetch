use actix_web::{delete, get, post, Scope};
use crate::utils::error::CustomError;

#[get("/")]
pub async fn get_watch_together() -> Result<(), CustomError> {
   Ok(())
}

#[post("/")]
pub async fn create_watch_together() -> Result<(), CustomError> {
    Ok(())
}

#[delete("/")]
pub async fn delete_watch_together() -> Result<(), CustomError> {
    Ok(())
}

pub fn routes() -> Scope {
    Scope::new("/watch-together")
        .service(get_watch_together)
        .service(create_watch_together)
}