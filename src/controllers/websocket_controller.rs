use actix::{ActorTryFutureExt, Addr};
use actix_web::{get, web::Data, web::Payload, Error, HttpResponse, HttpRequest, http};
use actix_web::web::Path;
use actix_web_actors::ws;
use uuid::Uuid;
use crate::controllers::web_socket::WsConn;
use crate::models::web_socket_message::Lobby;
use actix_web::http::header::{CONNECTION, HeaderValue, UPGRADE};

pub async fn start_connection(req: HttpRequest, stream: Payload)
    -> Result<HttpResponse, Error> {
    let ws = WsConn::new();
    println!("Websocket connection started");
    ws::start(ws, &req, stream)
}