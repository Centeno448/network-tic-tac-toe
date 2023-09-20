use futures_util::{SinkExt, StreamExt};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::{sleep, timeout};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use url::Url;
use uuid::Uuid;

use network_tic_tac_toe::configuration::get_configuration;
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

#[derive(Deserialize)]
pub struct MatchListResponse {
    pub category: String,
    pub body: MatchListResponseBody,
}

#[derive(Deserialize)]
pub struct MatchListResponseBody {
    pub matches: Vec<ResponseMatch>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseMatch {
    pub match_id: Uuid,
    pub room_name: String,
    pub players: String,
    pub status: String,
}

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

pub const START_MESSAGE: &'static str = r#"{ "message": "Start"}"#;
pub const LIST_MESSAGE: &'static str = r#"{ "message": "List"}"#;
pub const LEAVE_MESSAGE: &'static str = r#"{ "message": "Leave"}"#;

pub fn build_join_message(match_id: Uuid) -> String {
    format!(r#"{{ "message": "Join", "content": "{}"}}"#, match_id)
}

pub fn build_create_message(room: &str) -> String {
    format!(r#"{{ "message": "Create", "content": "{}"}}"#, room)
}

pub fn build_turn_message(turn: &str) -> String {
    format!(r#"{{ "message": "Turn", "content": "{}"}}"#, turn)
}

pub fn build_username_message(username: &str) -> String {
    format!(r#"{{ "message": "Username", "content": "{}"}}"#, username)
}

pub async fn process_message(socket: &mut WebSocketStream<MaybeTlsStream<TcpStream>>) -> Message {
    timeout(Duration::from_millis(10), socket.next())
        .await
        .unwrap()
        .unwrap()
        .expect("Failed to recieve message in under 10ms")
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
        .expect("Failed to send message.");
    sleep(Duration::from_millis(10)).await; // sleep to give time for server to process message
}

pub async fn setup_game(
    mut player_one: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    mut player_two: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
) {
    process_message(player_one).await; // Player 1 connects
    process_message(player_two).await; // Player 2 connects

    send_message(&mut player_one, &build_create_message("room")).await;

    process_message(&mut player_one).await;

    send_message(&mut player_two, LIST_MESSAGE).await;

    let player_two_response = process_message(&mut player_two).await;
    let player_two_response: MatchListResponse =
        serde_json::from_str(player_two_response.to_text().unwrap()).unwrap();

    let match_id = player_two_response.body.matches.first().unwrap().match_id;

    send_message(&mut player_two, &build_join_message(match_id)).await;

    process_message(&mut player_two).await;
    process_message(&mut player_one).await;
}

pub async fn join_room(
    mut existing_socket: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    mut joining_socket: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
) {
    send_message(&mut joining_socket, LIST_MESSAGE).await;

    let joining_socket_response = process_message(&mut joining_socket).await;
    let joining_socket_response: MatchListResponse =
        serde_json::from_str(joining_socket_response.to_text().unwrap()).unwrap();

    let match_id = joining_socket_response
        .body
        .matches
        .first()
        .unwrap()
        .match_id;

    send_message(&mut joining_socket, &build_join_message(match_id)).await;

    process_message(&mut joining_socket).await;
    process_message(&mut existing_socket).await;
}

pub async fn setup_and_start_game(
    mut player_one: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    mut player_two: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
) {
    setup_game(player_one, player_two).await;

    send_message(&mut player_one, START_MESSAGE).await; // Game start

    process_message(&mut player_one).await; // Player 1 recieves game start
    process_message(&mut player_two).await; // Player 2 recieves game start
}

pub async fn setup_game_for_tie(
    mut player_one: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    mut player_two: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
) {
    setup_and_start_game(player_one, player_two).await;

    send_message(&mut player_one, &build_turn_message("LL")).await; // Player 1 turn
    process_message(&mut player_two).await;
    send_message(&mut player_two, &build_turn_message("LM")).await; // Player 2 turn
    process_message(&mut player_one).await;

    send_message(&mut player_one, &build_turn_message("LR")).await; // Player 1 turn
    process_message(&mut player_two).await;
    send_message(&mut player_two, &build_turn_message("UL")).await; // Player 2 turn
    process_message(&mut player_one).await;

    send_message(&mut player_one, &build_turn_message("MM")).await; // Player 1 turn
    process_message(&mut player_two).await;
    send_message(&mut player_two, &build_turn_message("UR")).await; // Player 2 turn
    process_message(&mut player_one).await;

    send_message(&mut player_one, &build_turn_message("UM")).await; // Player 1 turn
    process_message(&mut player_two).await;
    send_message(&mut player_two, &build_turn_message("MR")).await; // Player 2 turn
    process_message(&mut player_one).await;
}

pub async fn setup_game_for_diagonal_victory(
    mut player_one: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    mut player_two: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
) {
    setup_and_start_game(player_one, player_two).await;

    send_message(&mut player_one, &build_turn_message("LL")).await; // Player 1 turn
    process_message(&mut player_two).await;
    send_message(&mut player_two, &build_turn_message("LM")).await; // Player 2 turn
    process_message(&mut player_one).await;

    send_message(&mut player_one, &build_turn_message("MM")).await; // Player 1 turn
    process_message(&mut player_two).await;
    send_message(&mut player_two, &build_turn_message("LR")).await; // Player 2 turn
    process_message(&mut player_one).await;
}

pub async fn setup_game_for_diagonal_mirror_victory(
    mut player_one: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    mut player_two: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
) {
    setup_and_start_game(player_one, player_two).await;

    send_message(&mut player_one, &build_turn_message("UL")).await; // Player 1 turn
    process_message(&mut player_two).await;
    send_message(&mut player_two, &build_turn_message("LM")).await; // Player 2 turn
    process_message(&mut player_one).await;

    send_message(&mut player_one, &build_turn_message("MM")).await; // Player 1 turn
    process_message(&mut player_two).await;
    send_message(&mut player_two, &build_turn_message("LL")).await; // Player 2 turn
    process_message(&mut player_one).await;
}

pub async fn setup_game_for_cross_victory(
    mut player_one: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    mut player_two: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
) {
    setup_and_start_game(player_one, player_two).await;

    send_message(&mut player_one, &build_turn_message("LL")).await; // Player 1 turn
    process_message(&mut player_two).await;
    send_message(&mut player_two, &build_turn_message("UL")).await; // Player 2 turn
    process_message(&mut player_one).await;

    send_message(&mut player_one, &build_turn_message("LM")).await; // Player 1 turn
    process_message(&mut player_two).await;
    send_message(&mut player_two, &build_turn_message("UM")).await; // Player 2 turn
    process_message(&mut player_one).await;
}

pub async fn setup_game_for_circle_victory(
    mut player_one: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    mut player_two: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
) {
    setup_and_start_game(player_one, player_two).await;

    send_message(&mut player_one, &build_turn_message("LL")).await; // Player 1 turn
    process_message(&mut player_two).await;
    send_message(&mut player_two, &build_turn_message("LR")).await; // Player 2 turn
    process_message(&mut player_one).await;

    send_message(&mut player_one, &build_turn_message("MM")).await; // Player 1 turn
    process_message(&mut player_two).await;
    send_message(&mut player_two, &build_turn_message("MR")).await; // Player 2 turn
    process_message(&mut player_one).await;

    send_message(&mut player_one, &build_turn_message("ML")).await; // Player 1 turn
    process_message(&mut player_two).await;
}
