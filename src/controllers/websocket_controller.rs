use std::ptr::null;
use actix::Addr;
use actix_web::{web::Payload, Error, HttpResponse, HttpRequest, web, web::Data, get, Responder};
use actix_web_actors::ws;
use uuid::Uuid;
use crate::controllers::web_socket::WsConn;
use crate::models::messages::{BroadcastMessage, ClientActorMessage};
use crate::models::web_socket_message::Lobby;

#[get("/ws")]
pub async fn start_connection(req: HttpRequest, stream: Payload, lobby: Data<Addr<Lobby>>)
    -> Result<HttpResponse, Error> {
    let ws = WsConn::new( lobby.get_ref().clone());
    let resp = ws::start(ws, &req, stream)?;
    Ok(resp)
}

#[get("/ws/send")]
pub async fn send_message_to_user(lobby: Data<Addr<Lobby>>) ->impl Responder{
    lobby.get_ref().send(BroadcastMessage { type_of: "".to_string(), message: "Broadcast".to_string(),
        podcast: None,
        podcast_episodes: None,
        podcast_episode: None }).await.unwrap();

    HttpResponse::Ok().body("Message sent")
}