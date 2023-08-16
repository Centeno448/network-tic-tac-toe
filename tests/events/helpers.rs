use actix::{Actor, Context, Handler};
use network_tic_tac_toe::game_server::domain::{TeamSymbol, TurnMove};
use network_tic_tac_toe::game_server::{GameRoomStatus, GameServer, ServerMessage};
use network_tic_tac_toe::telemetry::{get_subscriber, init_subscriber};
use once_cell::sync::Lazy;
use std::sync::{atomic::AtomicUsize, Arc};
use uuid::Uuid;

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct MockPlayerSession;

impl MockPlayerSession {
    pub fn new() -> Self {
        MockPlayerSession
    }
}

impl Actor for MockPlayerSession {
    type Context = Context<Self>;
}

impl Handler<ServerMessage> for MockPlayerSession {
    type Result = ();
    fn handle(&mut self, _: ServerMessage, _: &mut Self::Context) -> Self::Result {}
}

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

pub fn setup_empty_game_server() -> GameServer {
    Lazy::force(&TRACING);
    let visitors = Arc::new(AtomicUsize::new(0));
    GameServer::new(visitors)
}

pub fn setup_game_server_with_status(status: GameRoomStatus) -> GameServer {
    let mut server = setup_empty_game_server();

    server.rooms.get_mut("lobby").unwrap().status = status;
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

pub fn setup_game_for_tie(game_server: &mut GameServer, player_one: Uuid, player_two: Uuid) {
    let room = game_server.rooms.get_mut("lobby").unwrap();

    room.moves_made.insert(TurnMove::LL, player_one);
    room.moves_made.insert(TurnMove::LR, player_one);
    room.moves_made.insert(TurnMove::MM, player_one);
    room.moves_made.insert(TurnMove::UM, player_one);
    room.moves_made.insert(TurnMove::LM, player_two);
    room.moves_made.insert(TurnMove::UL, player_two);
    room.moves_made.insert(TurnMove::UR, player_two);
    room.moves_made.insert(TurnMove::MR, player_two);
}

pub fn setup_game_for_diagonal_victory(
    game_server: &mut GameServer,
    player_one: &Uuid,
    player_two: &Uuid,
) {
    let room = game_server.rooms.get_mut("lobby").unwrap();

    room.moves_made.insert(TurnMove::LL, player_one.clone());
    room.moves_made.insert(TurnMove::MM, player_one.clone());

    room.moves_made.insert(TurnMove::LM, player_two.clone());
    room.moves_made.insert(TurnMove::LR, player_two.clone());
}

pub fn setup_game_for_cross_victory(
    game_server: &mut GameServer,
    player_one: &Uuid,
    player_two: &Uuid,
) {
    let room = game_server.rooms.get_mut("lobby").unwrap();

    room.moves_made.insert(TurnMove::LL, player_one.clone());
    room.moves_made.insert(TurnMove::LM, player_one.clone());

    room.moves_made.insert(TurnMove::UL, player_two.clone());
    room.moves_made.insert(TurnMove::UM, player_two.clone());
}

pub fn setup_game_for_circle_victory(
    game_server: &mut GameServer,
    player_one: &Uuid,
    player_two: &Uuid,
) {
    let room = game_server.rooms.get_mut("lobby").unwrap();

    room.current_turn = TeamSymbol::Circle;

    room.moves_made.insert(TurnMove::LL, player_one.clone());
    room.moves_made.insert(TurnMove::MM, player_one.clone());
    room.moves_made.insert(TurnMove::ML, player_one.clone());

    room.moves_made.insert(TurnMove::LR, player_two.clone());
    room.moves_made.insert(TurnMove::MR, player_two.clone());
}
