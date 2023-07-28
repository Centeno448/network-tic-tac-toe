use actix::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    sync::{atomic::AtomicUsize, Arc},
};
use uuid::Uuid;

use super::domain::TurnMove;
use crate::game_server::domain::TeamSymbol;

#[derive(Message)]
#[rtype(result = "()")]
pub struct ServerMessage(pub String);

#[derive(Debug)]
pub struct GameServer {
    pub sessions: HashMap<Uuid, Recipient<ServerMessage>>,
    pub rooms: HashMap<String, GameRoom>,
    pub visitor_count: Arc<AtomicUsize>,
}

#[derive(Debug)]
pub struct GameRoom {
    pub players: HashSet<Uuid>,
    pub status: GameRoomStatus,
    pub current_turn: TeamSymbol,
    pub moves_made: HashMap<TurnMove, Uuid>,
}

impl GameRoom {
    pub fn new() -> Self {
        GameRoom {
            players: HashSet::new(),
            status: GameRoomStatus::Waiting,
            current_turn: TeamSymbol::Cross,
            moves_made: HashMap::new(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum GameRoomStatus {
    Waiting,
    Started,
}

impl GameServer {
    pub fn new(visitor_count: Arc<AtomicUsize>) -> GameServer {
        // default room
        let mut rooms = HashMap::new();
        rooms.insert("lobby".to_owned(), GameRoom::new());

        GameServer {
            sessions: HashMap::new(),
            rooms,
            visitor_count,
        }
    }
}

impl GameServer {
    /// Relay message to everyone else in the room
    pub fn send_message(&self, room: &str, message: &str, skip_id: Uuid) {
        if let Some(game_room) = self.rooms.get(room) {
            for id in game_room.players.iter() {
                if *id != skip_id {
                    if let Some(addr) = self.sessions.get(id) {
                        addr.do_send(ServerMessage(message.to_owned()));
                    }
                }
            }
        }
    }

    /// Send message to all users in the room
    pub fn send_message_all(&self, room: &str, message: &str) {
        if let Some(game_room) = self.rooms.get(room) {
            for id in game_room.players.iter() {
                if let Some(addr) = self.sessions.get(id) {
                    addr.do_send(ServerMessage(message.to_owned()));
                }
            }
        }
    }

    /// Send message to specific user
    pub fn send_direct_message(&self, addr: &Recipient<ServerMessage>, message: &str) {
        addr.do_send(ServerMessage(message.to_owned()));
    }
}

impl Actor for GameServer {
    type Context = Context<Self>;
}
