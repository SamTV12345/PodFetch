use actix::Addr;
use actix_web::{web::Payload, HttpResponse, Error, HttpRequest, web::Data, get};
use actix_web_actors::ws;
use crate::controllers::web_socket::WsConn;
use crate::models::web_socket_message::Lobby;

#[get("/ws")]
pub async fn start_connection(req: HttpRequest, stream: Payload, lobby: Data<Addr<Lobby>>)
                              -> Result<HttpResponse, Error> {
    let ws = WsConn::new( lobby.get_ref().clone());
    let resp = ws::start(ws, &req, stream)?;
    Ok(resp)
}
