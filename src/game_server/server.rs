use crate::game_server::{domain::TeamSymbol, CommandCategory, Commmand};
use actix::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use uuid::Uuid;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

#[derive(Message)]
#[rtype(result = "TeamSymbol")]
pub struct Connect {
    pub id: Uuid,
    pub addr: Recipient<Message>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct StartGame {
    pub id: Uuid,
}

#[derive(Debug)]
pub struct GameServer {
    pub sessions: HashMap<Uuid, Recipient<Message>>,
    pub rooms: HashMap<String, GameRoom>,
    pub visitor_count: Arc<AtomicUsize>,
}

#[derive(Debug)]
pub struct GameRoom {
    pub players: HashSet<Uuid>,
    pub status: GameRoomStatus,
    pub current_turn: TeamSymbol,
}

impl GameRoom {
    pub fn new() -> Self {
        GameRoom {
            players: HashSet::new(),
            status: GameRoomStatus::Waiting,
            current_turn: TeamSymbol::Cross,
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
                        addr.do_send(Message(message.to_owned()));
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
                    addr.do_send(Message(message.to_owned()));
                }
            }
        }
    }

    /// Send message to specific user
    pub fn send_direct_message(&self, addr: &Recipient<Message>, message: &str) {
        addr.do_send(Message(message.to_owned()));
    }
}

impl Actor for GameServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for GameServer {
    type Result = TeamSymbol;

    #[tracing::instrument(name = "Player connect", skip_all, fields(player_session_id=%msg.id))]
    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        let id = msg.id;

        self.rooms
            .entry("lobby".to_owned())
            .or_insert_with(GameRoom::new)
            .players
            .insert(id);

        let _ = self.visitor_count.fetch_add(1, Ordering::SeqCst);
        let count = self.visitor_count.load(Ordering::Relaxed);
        tracing::info!("Number of players in lobby: {count}");

        let player_symbol = if self.rooms.get("lobby").unwrap().players.len() > 1 {
            TeamSymbol::Circle
        } else {
            TeamSymbol::Cross
        };

        let connect_command = Commmand::new_serialized(CommandCategory::Connected, &player_symbol);

        self.send_direct_message(&msg.addr, &connect_command);

        let command =
            Commmand::new_serialized(CommandCategory::PlayerConnected, msg.id.to_string());
        self.send_message("lobby", &command, id);

        self.sessions.insert(id, msg.addr);

        player_symbol
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
            for (_, room) in &mut self.rooms {
                room.players.remove(&msg.id);
            }
        }

        let _ = self.visitor_count.fetch_sub(1, Ordering::SeqCst);
        let count = self.visitor_count.load(Ordering::Relaxed);
        tracing::info!("Number of players in lobby: {count}");

        let command = Commmand::new(CommandCategory::PlayerDisconnected, msg.id.to_string());

        let result = serde_json::to_string(&command).unwrap_or("".into());

        self.send_message("lobby", &result, msg.id);
    }
}

impl Handler<StartGame> for GameServer {
    type Result = ();

    #[tracing::instrument(name = "Game Start", skip_all, fields(room_name, player_id=%msg.id))]
    fn handle(&mut self, msg: StartGame, _: &mut Self::Context) -> Self::Result {
        let mut result_room: Option<String> = None;

        for (room_name, room) in self
            .rooms
            .iter_mut()
            .filter(|(_, room)| room.status == GameRoomStatus::Waiting)
        {
            if room.players.contains(&msg.id) {
                tracing::Span::current().record("room_name", room_name);
                room.status = GameRoomStatus::Started;

                result_room = Some(room_name.clone());
                break;
            } else {
                tracing::info!("Player is not in any room with status waiting.");
            }
        }

        if let Some(room_name) = result_room {
            let command = Commmand::new_serialized(CommandCategory::GameStart, "");

            self.send_message_all(&room_name, &command);
        }
    }
}
