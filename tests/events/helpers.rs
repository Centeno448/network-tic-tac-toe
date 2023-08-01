use network_tic_tac_toe::game_server::{GameRoomStatus, GameServer};
use std::sync::{atomic::AtomicUsize, Arc};
use uuid::Uuid;

pub fn get_player_ids_from_room(game_server: &GameServer, room_name: &str) -> Vec<Uuid> {
    game_server
        .rooms
        .get(room_name)
        .unwrap()
        .players
        .clone()
        .into_iter()
        .collect()
}

pub fn setup_game_server() -> GameServer {
    let visitors = Arc::new(AtomicUsize::new(2));
    let mut server = GameServer::new(visitors);

    server.rooms.get_mut("lobby").unwrap().status = GameRoomStatus::Started;
    server
        .rooms
        .get_mut("lobby")
        .unwrap()
        .players
        .insert(Uuid::new_v4());
    server
        .rooms
        .get_mut("lobby")
        .unwrap()
        .players
        .insert(Uuid::new_v4());

    server
}
