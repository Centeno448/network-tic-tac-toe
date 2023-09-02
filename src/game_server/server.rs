use actix::dev::{MessageResponse, OneshotSender};
use actix::prelude::{Actor, Context, Message, Recipient};
use serde::Serialize;
use std::{
    collections::HashMap,
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
    pub rooms: HashMap<Uuid, GameRoom>,
    pub visitor_count: Arc<AtomicUsize>,
}

#[derive(Debug, Clone)]
pub struct GameRoom {
    pub players: HashMap<Uuid, String>,
    pub status: GameRoomStatus,
    pub current_turn: TeamSymbol,
    pub name: String,
    pub moves_made: HashMap<TurnMove, Uuid>,
}

impl GameRoom {
    pub fn new(name: String) -> Self {
        GameRoom {
            players: HashMap::new(),
            status: GameRoomStatus::Waiting,
            current_turn: TeamSymbol::Cross,
            name,
            moves_made: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameState(pub Option<GameRoom>);

impl<A, M> MessageResponse<A, M> for GameState
where
    A: Actor,
    M: Message<Result = GameState>,
{
    fn handle(self, _: &mut A::Context, tx: Option<OneshotSender<M::Result>>) {
        if let Some(tx) = tx {
            let _ = tx.send(self);
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum GameRoomStatus {
    Waiting,
    Started,
    Finished,
}

impl GameServer {
    pub fn new(visitor_count: Arc<AtomicUsize>) -> GameServer {
        let rooms = HashMap::new();

        GameServer {
            sessions: HashMap::new(),
            rooms,
            visitor_count,
        }
    }
}

impl GameServer {
    /// Relay message to everyone else in the room
    pub fn send_message(&self, room: &Uuid, message: &str, skip_id: Uuid) {
        if let Some(game_room) = self.rooms.get(room) {
            for (id, _) in game_room.players.iter() {
                if *id != skip_id {
                    if let Some(addr) = self.sessions.get(id) {
                        addr.do_send(ServerMessage(message.to_owned()));
                    }
                }
            }
        }
    }

    /// Send message to all users in the room
    pub fn send_message_all(&self, room: &Uuid, message: &str) {
        if let Some(game_room) = self.rooms.get(room) {
            for (id, _) in game_room.players.iter() {
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
