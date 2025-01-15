use actix_web::web;
use crate::adapters::audiobookshelf::login::{login_audiobookshelf, login_audiobookshelf_redundant};
use crate::adapters::audiobookshelf::ping::ping;
use crate::adapters::audiobookshelf::status::get_status;
use crate::adapters::audiobookshelf::ws::handle_socket_io;

pub fn add_audiobookshelf_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_status)
        .service(login_audiobookshelf)
        .service(login_audiobookshelf_redundant)
        .service(ping)
        .service(handle_socket_io);
}