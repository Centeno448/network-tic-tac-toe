use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use crate::game_server::{CommandCategory, Commmand};
use actix::prelude::*;
use uuid::Uuid;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub id: Uuid,
    pub addr: Recipient<Message>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: Uuid,
}

#[derive(Debug)]
pub struct GameServer {
    sessions: HashMap<Uuid, Recipient<Message>>,
    rooms: HashMap<String, HashSet<Uuid>>,
    visitor_count: Arc<AtomicUsize>,
}

impl GameServer {
    pub fn new(visitor_count: Arc<AtomicUsize>) -> GameServer {
        // default room
        let mut rooms = HashMap::new();
        rooms.insert("lobby".to_owned(), HashSet::new());

        GameServer {
            sessions: HashMap::new(),
            rooms,
            visitor_count,
        }
    }
}

impl GameServer {
    /// Send message to all users in the room
    fn send_message(&self, room: &str, message: &str, skip_id: Uuid) {
        if let Some(sessions) = self.rooms.get(room) {
            for id in sessions {
                if *id != skip_id {
                    if let Some(addr) = self.sessions.get(id) {
                        addr.do_send(Message(message.to_owned()));
                    }
                }
            }
        }
    }
}

impl Actor for GameServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for GameServer {
    type Result = ();

    #[tracing::instrument(name = "Player connect", skip_all, fields(player_session_id=%msg.id))]
    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        let id = msg.id;
        self.sessions.insert(id, msg.addr);

        // auto join session to main room
        self.rooms
            .entry("lobby".to_owned())
            .or_insert_with(HashSet::new)
            .insert(id);

        let _ = self.visitor_count.fetch_add(1, Ordering::SeqCst);
        let count = self.visitor_count.load(Ordering::Relaxed);
        tracing::info!("Number of players in lobby: {count}");

        let command = Commmand {
            category: CommandCategory::PlayerConnected,
            body: msg.id.to_string(),
        };

        let result = serde_json::to_string(&command).unwrap_or("".into());

        self.send_message("lobby", &result, id);
    }
}

impl Handler<Disconnect> for GameServer {
    type Result = ();

    #[tracing::instrument(
        name = "Player disconnect",
        skip_all,
        fields(player_session_id=%msg.id)
    )]
    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) -> Self::Result {
        if self.sessions.remove(&msg.id).is_some() {
            for (_, sessions) in &mut self.rooms {
                sessions.remove(&msg.id);
            }
        }

        let _ = self.visitor_count.fetch_sub(1, Ordering::SeqCst);
        let count = self.visitor_count.load(Ordering::Relaxed);
        tracing::info!("Number of players in lobby: {count}");
    }
}
