use crate::constants::inner_constants::MAIN_ROOM;
use rand::random;
use std::collections::{HashMap, HashSet};
use std::io;
use tokio::sync::{mpsc, oneshot};

type RoomId = String;
pub type ConnId = usize;

pub type Msg = String;

/// A command received by the [`ChatServer`].
#[derive(Debug)]
enum Command {
    Broadcast {
        room: RoomId,
        msg: Msg,
        res_tx: Option<oneshot::Sender<()>>,
    },
    Connect {
        conn_tx: mpsc::UnboundedSender<Msg>,
        res_tx: oneshot::Sender<ConnId>,
    },

    Disconnect {
        conn: ConnId,
    },
    Message {
        msg: Msg,
        conn: ConnId,
        res_tx: oneshot::Sender<()>,
    },
}

/// A multiroom chat server.
///
/// Contains the logic of how connections chat with each other plus room management.
///
/// Call and spawn [`run`](Self::run) to start processing commands.
#[derive(Debug)]
pub struct ChatServer {
    /// Map of connection IDs to their message receivers.
    sessions: HashMap<ConnId, mpsc::UnboundedSender<Msg>>,

    /// Map of room name to participant IDs in that room.
    rooms: HashMap<RoomId, HashSet<ConnId>>,

    /// Command receiver.
    cmd_rx: mpsc::UnboundedReceiver<Command>,
}

impl ChatServer {
    pub fn new() -> (Self, ChatServerHandle) {
        // create empty server
        let mut rooms = HashMap::with_capacity(4);

        // create default room
        rooms.insert(MAIN_ROOM.parse().unwrap(), HashSet::new());

        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();

        (
            Self {
                sessions: HashMap::new(),
                rooms,
                //visitor_count: Arc::new(AtomicUsize::new(0)),
                cmd_rx,
            },
            ChatServerHandle { cmd_tx },
        )
    }

    /// Send message to specific room, also to the user itself
    async fn send_broadcast_to_room(&self, room_id: &str, msg: impl Into<Msg>) {
        if let Some(sessions) = self.rooms.get(room_id) {
            let msg = msg.into();

            for conn_id in sessions {
                if let Some(tx) = self.sessions.get(conn_id) {
                    // errors if client disconnected abruptly and hasn't been timed-out yet
                    let _ = tx.send(msg.clone());
                }
            }
        }
    }

    /// Send message to users in a room.
    ///
    /// `skip` is used to prevent messages triggered by a connection also being received by it.
    async fn send_system_message(&self, room: &str, skip: ConnId, msg: impl Into<Msg>) {
        if let Some(sessions) = self.rooms.get(room) {
            let msg = msg.into();

            for conn_id in sessions {
                if *conn_id != skip {
                    if let Some(tx) = self.sessions.get(conn_id) {
                        // errors if client disconnected abruptly and hasn't been timed-out yet
                        let _ = tx.send(msg.clone());
                    }
                }
            }
        }
    }

    /// Send message to all other users in current room.
    ///
    /// `conn` is used to find current room and prevent messages sent by a connection also being
    /// received by it.
    async fn send_message(&self, conn: ConnId, msg: impl Into<Msg>) {
        if let Some(room) = self
            .rooms
            .iter()
            .find_map(|(room, participants)| participants.contains(&conn).then_some(room))
        {
            self.send_system_message(room, conn, msg).await;
        };
    }

    /// Register new session and assign unique ID to this session
    async fn connect(&mut self, tx: mpsc::UnboundedSender<Msg>) -> ConnId {
        log::info!("Someone joined");

        // notify all users in same room
        //self.send_system_message("main", 0, "Someone joined").await;

        // register session with random connection ID
        let id = random::<ConnId>();
        self.sessions.insert(id, tx);

        // auto join session to main room
        self.rooms
            .entry(MAIN_ROOM.parse().unwrap())
            .or_default()
            .insert(id);
        log::info!("Joined main room");

        //let count = self.visitor_count.fetch_add(1, Ordering::SeqCst);
        /*self.send_system_message("main", 0, format!("Total visitors {count}"))
        .await;*/

        // send id back
        id
    }

    /// Unregister connection from room map and broadcast disconnection message.
    async fn disconnect(&mut self, conn_id: ConnId) {
        println!("Someone disconnected");

        let mut rooms: Vec<RoomId> = Vec::new();

        // remove sender
        if self.sessions.remove(&conn_id).is_some() {
            // remove session from all rooms
            for (name, sessions) in &mut self.rooms {
                if sessions.remove(&conn_id) {
                    rooms.push(name.to_owned());
                }
            }
        }

        // send message to other users
        for room in rooms {
            self.send_system_message(&room, 0, "Someone disconnected")
                .await;
        }
    }

    pub async fn run(mut self) -> io::Result<()> {
        while let Some(cmd) = self.cmd_rx.recv().await {
            match cmd {
                Command::Broadcast { msg, room, res_tx } => {
                    let res = self.send_broadcast_to_room(&room, msg).await;

                    if let Some(res_tx) = res_tx {
                        let _ = res_tx.send(res);
                    }
                }
                Command::Connect { conn_tx, res_tx } => {
                    let conn_id = self.connect(conn_tx).await;
                    let _ = res_tx.send(conn_id);
                }
                Command::Disconnect { conn } => {
                    self.disconnect(conn).await;
                }
                Command::Message { conn, msg, res_tx } => {
                    self.send_message(conn, msg).await;
                    let _ = res_tx.send(());
                }
            }
        }

        Ok(())
    }
}

/// Handle and command sender for chat server.
///
/// Reduces boilerplate of setting up response channels in WebSocket handlers.
#[derive(Debug, Clone)]
pub struct ChatServerHandle {
    cmd_tx: mpsc::UnboundedSender<Command>,
}

impl ChatServerHandle {
    /// Register client message sender and obtain connection ID.
    pub async fn connect(&self, conn_tx: mpsc::UnboundedSender<Msg>) -> ConnId {
        let (res_tx, res_rx) = oneshot::channel();

        // unwrap: chat server should not have been dropped
        self.cmd_tx
            .send(Command::Connect { conn_tx, res_tx })
            .unwrap();

        // unwrap: chat server does not drop out response channel
        res_rx.await.unwrap()
    }

    pub async fn send_broadcast(&self, room_id: RoomId, msg: impl Into<Msg>) {
        let (res_tx, res_rx) = oneshot::channel();

        self.cmd_tx
            .send(Command::Broadcast {
                msg: msg.into(),
                room: room_id,
                res_tx: Some(res_tx),
            })
            .unwrap();

        // unwrap: chat server does not drop our response channel
        res_rx.await.unwrap();
    }

    pub fn send_broadcast_sync(&self, room_id: RoomId, msg: impl Into<Msg>) {
        self.cmd_tx
            .send(Command::Broadcast {
                msg: msg.into(),
                room: room_id,
                res_tx: None,
            })
            .unwrap();
    }

    /// Broadcast message to current room.
    pub async fn send_message(&self, conn: ConnId, msg: impl Into<Msg>) {
        let (res_tx, res_rx) = oneshot::channel();

        // unwrap: chat server should not have been dropped
        self.cmd_tx
            .send(Command::Message {
                msg: msg.into(),
                conn,
                res_tx,
            })
            .unwrap();

        // unwrap: chat server does not drop our response channel
        res_rx.await.unwrap();
    }

    /// Unregister message sender and broadcast disconnection message to current room.
    pub fn disconnect(&self, conn: ConnId) {
        // unwrap: chat server should not have been dropped
        self.cmd_tx.send(Command::Disconnect { conn }).unwrap();
    }
}
