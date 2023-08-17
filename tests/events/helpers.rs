use futures_util::{FutureExt, SinkExt, StreamExt};
use once_cell::sync::Lazy;
use std::sync::{atomic::AtomicUsize, Arc};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use url::Url;
use uuid::Uuid;

use network_tic_tac_toe::configuration::get_configuration;
use network_tic_tac_toe::game_server::domain::{TeamSymbol, TurnMove};
use network_tic_tac_toe::game_server::{GameRoomStatus, GameServer};
use network_tic_tac_toe::startup::Application;
use network_tic_tac_toe::telemetry::{get_subscriber, init_subscriber};

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

pub struct TestApp {
    pub address: String,
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        // use random port
        c.port = 0;

        c
    };

    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application");

    let address = format!("ws://{}:{}", configuration.host, application.port());
    let _ = tokio::spawn(application.run_until_stopped());

    TestApp { address }
}

impl TestApp {
    pub async fn connect_player(&self) -> WebSocketStream<MaybeTlsStream<TcpStream>> {
        let (socket, _) = connect_async(Url::parse(&self.address).unwrap())
            .await
            .expect("Failed to connect");

        socket
    }
}

pub async fn process_message(socket: &mut WebSocketStream<MaybeTlsStream<TcpStream>>) -> Message {
    socket
        .next()
        .await
        .expect("Failed to fetch response")
        .unwrap()
}

pub async fn process_message_result(
    socket: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
) -> Option<Result<Message, tokio_tungstenite::tungstenite::Error>> {
    timeout(Duration::from_millis(10), socket.next())
        .await
        .unwrap_or_else(|_| None)
}

pub async fn send_message(socket: &mut WebSocketStream<MaybeTlsStream<TcpStream>>, msg: &str) {
    socket
        .send(Message::Text(msg.into()))
        .await
        .expect("Failed to send message.")
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
