use futures_util::{SinkExt, StreamExt};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;
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
pub struct ResponseMatch {
    pub match_id: Uuid,
    pub room_name: String,
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

pub async fn setup_and_start_game(
    mut player_one: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    mut player_two: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
) {
    process_message(&mut player_one).await; // Player 1 connects
    process_message(&mut player_two).await; // Player 2 connects

    send_message(&mut player_one, "/create_match room").await;

    process_message(&mut player_one).await;

    send_message(&mut player_two, "/list_matches").await;

    let player_two_response = process_message(&mut player_two).await;
    let player_two_response: MatchListResponse =
        serde_json::from_str(player_two_response.to_text().unwrap()).unwrap();

    let match_id = player_two_response.body.matches.first().unwrap().match_id;

    send_message(&mut player_two, &format!("/join_match {}", match_id)).await;

    process_message(&mut player_two).await;
    process_message(&mut player_one).await;

    send_message(&mut player_one, "/start").await; // Game start

    process_message(&mut player_one).await; // Player 1 recieves game start
    process_message(&mut player_two).await; // Player 2 recieves game start
}

pub async fn setup_game_for_tie(
    mut player_one: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    mut player_two: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
) {
    setup_and_start_game(player_one, player_two).await;

    send_message(&mut player_one, "/turn LL").await; // Player 1 turn
    process_message(&mut player_two).await;
    send_message(&mut player_two, "/turn LM").await; // Player 2 turn
    process_message(&mut player_one).await;

    send_message(&mut player_one, "/turn LR").await; // Player 1 turn
    process_message(&mut player_two).await;
    send_message(&mut player_two, "/turn UL").await; // Player 2 turn
    process_message(&mut player_one).await;

    send_message(&mut player_one, "/turn MM").await; // Player 1 turn
    process_message(&mut player_two).await;
    send_message(&mut player_two, "/turn UR").await; // Player 2 turn
    process_message(&mut player_one).await;

    send_message(&mut player_one, "/turn UM").await; // Player 1 turn
    process_message(&mut player_two).await;
    send_message(&mut player_two, "/turn MR").await; // Player 2 turn
    process_message(&mut player_one).await;
}

pub async fn setup_game_for_diagonal_victory(
    mut player_one: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    mut player_two: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
) {
    setup_and_start_game(player_one, player_two).await;

    send_message(&mut player_one, "/turn LL").await; // Player 1 turn
    process_message(&mut player_two).await;
    send_message(&mut player_two, "/turn LM").await; // Player 2 turn
    process_message(&mut player_one).await;

    send_message(&mut player_one, "/turn MM").await; // Player 1 turn
    process_message(&mut player_two).await;
    send_message(&mut player_two, "/turn LR").await; // Player 2 turn
    process_message(&mut player_one).await;
}

pub async fn setup_game_for_cross_victory(
    mut player_one: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    mut player_two: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
) {
    setup_and_start_game(player_one, player_two).await;

    send_message(&mut player_one, "/turn LL").await; // Player 1 turn
    process_message(&mut player_two).await;
    send_message(&mut player_two, "/turn UL").await; // Player 2 turn
    process_message(&mut player_one).await;

    send_message(&mut player_one, "/turn LM").await; // Player 1 turn
    process_message(&mut player_two).await;
    send_message(&mut player_two, "/turn UM").await; // Player 2 turn
    process_message(&mut player_one).await;
}

pub async fn setup_game_for_circle_victory(
    mut player_one: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    mut player_two: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
) {
    setup_and_start_game(player_one, player_two).await;

    send_message(&mut player_one, "/turn LL").await; // Player 1 turn
    process_message(&mut player_two).await;
    send_message(&mut player_two, "/turn LR").await; // Player 2 turn
    process_message(&mut player_one).await;

    send_message(&mut player_one, "/turn MM").await; // Player 1 turn
    process_message(&mut player_two).await;
    send_message(&mut player_two, "/turn MR").await; // Player 2 turn
    process_message(&mut player_one).await;

    send_message(&mut player_one, "/turn ML").await; // Player 1 turn
    process_message(&mut player_two).await;
}
