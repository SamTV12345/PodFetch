use crate::models::messages::{BroadcastMessage, Connect, Disconnect, WsMessage};
use actix::prelude::{Message, Recipient};
use actix::{Actor, Context, Handler};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Message)]
#[rtype(result = "()")]
pub struct SocketMessage {
    pub message_type: String,
    pub message: String,
    pub timestamp: String,
}

type Socket = Recipient<WsMessage>;

#[derive(Default)]
pub struct Lobby {
    sessions: HashMap<Uuid, Socket>,
}



impl Handler<BroadcastMessage> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: BroadcastMessage, _: &mut Context<Self>) {
        self.sessions.clone().into_values().for_each(|socket| {
            log::debug!("Sending message to socket: {}", msg.message);
            socket.do_send(WsMessage(json!(msg).to_string()));
        });
    }
}

impl Actor for Lobby {
    type Context = Context<Self>;
}

impl Handler<Disconnect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        if self.sessions.remove(&msg.id).is_some() {
            self.sessions.clone().into_values().for_each(|_| {
                log::debug!("Disconnected web client");
            });
        }
    }
}

impl Handler<Connect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        self.sessions.insert(msg.self_id, msg.addr);
    }
}
