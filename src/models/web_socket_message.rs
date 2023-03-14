use std::collections::{HashMap};
use actix::{Actor, Context, Handler};
use actix::prelude::{Message, Recipient};
use log::log;
use serde_json::json;
use uuid::Uuid;
use crate::models::messages::{BroadcastMessage, Connect, Disconnect, WsMessage};

#[derive(Message)]
#[rtype(result = "()")]
pub struct SocketMessage {
    pub message_type: String,
    pub message: String,
    pub timestamp: String
}


type Socket = Recipient<WsMessage>;

pub struct Lobby {
    sessions: HashMap<Uuid, Socket>
}

impl Default for Lobby {
    fn default() -> Lobby {
        Lobby {
            sessions: HashMap::new()
        }
    }
}

impl Handler<BroadcastMessage> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: BroadcastMessage, _: &mut Context<Self>) {
        self.sessions.clone().into_values().for_each(|socket| {
            println!("Sending message to socket: {}", msg.message);
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
            self.sessions.clone().into_values().for_each( |_|{
                log::debug!("Disconnected web client");
            });
        }
    }
}

impl Handler<Connect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        self.sessions.insert(
            msg.self_id,
            msg.addr,
        );
    }
}