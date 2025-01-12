use actix_web::web;
use crate::adapters::audiobookshelf::login::login_audiobookshelf;
use crate::adapters::audiobookshelf::status::get_status;

pub fn add_audiobookshelf_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_status)
        .service(login_audiobookshelf);
}