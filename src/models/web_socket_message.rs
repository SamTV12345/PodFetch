use std::collections::{HashMap};
use actix::{Actor, Context, Handler};
use actix::prelude::{Message, Recipient};
use actix_web::web;
use serde::Serialize;
use serde_json::json;
use uuid::Uuid;
use crate::models::messages::{BroadcastMessage, ClientActorMessage, Connect, Disconnect, WsMessage};

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

impl Lobby {
    fn send_message(&self, message: &str, id_to: &Uuid) {
        if let Some(socket_recipient) = self.sessions.get(id_to) {
            let _ = socket_recipient
                .do_send(WsMessage(message.to_owned()));
        } else {
            println!("attempting to send message but couldn't find user id.");
        }
    }

    pub fn get_all_sockets(&self) -> Vec<Socket> {
        self.sessions.clone().into_values().collect()
    }

    pub fn broadcast_message(&self, message: &str) {

    }
}

impl Handler<BroadcastMessage> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: BroadcastMessage, _: &mut Context<Self>) {
        println!("Handling broadcast message");
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
        println!("Disconnecting");
        println!("Handling disconnect");
        println!("Sessions length: {}", self.sessions.len().to_string());
        if self.sessions.remove(&msg.id).is_some() {
            self.sessions.clone().into_values().for_each(|socket| {
                socket.do_send(WsMessage("Test123".to_string()));
            });
        }
    }
}

impl Handler<Connect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {

        println!("Inserting new connection");
        self.sessions.insert(
            msg.self_id,
            msg.addr,
        );
        println!("Sessions length: {}", self.sessions.len().to_string());
        self.send_message(&format!("your id is {}", msg.self_id), &msg.self_id)
    }
}

impl Handler<ClientActorMessage> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: ClientActorMessage, _ctx: &mut Context<Self>) -> Self::Result {
        if msg.msg.starts_with("\\w") {
            if let Some(id_to) = msg.msg.split(' ').collect::<Vec<&str>>().get(1) {
                self.send_message(&msg.msg, &Uuid::parse_str(id_to).unwrap())
            }
        } else {
            print!("sending message to all users");
        }
    }
}