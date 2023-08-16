use actix::Actor;
use uuid::Uuid;

use network_tic_tac_toe::game_server::domain::TeamSymbol;
use network_tic_tac_toe::game_server::events::{GetGameState, StartGame};
use network_tic_tac_toe::game_server::GameRoomStatus;

use crate::helpers::{get_player_ids_from_room, setup_game_server_with_status};

#[actix_web::test]
async fn player_outside_room_cant_start_game() {
    // Arrange
    let server = setup_game_server_with_status(GameRoomStatus::Waiting);
    let server_addr = server.start();

    let start = StartGame {
        id: Uuid::new_v4(),
        team_symbol: Some(TeamSymbol::Cross),
    };

    // Act
    let _ = server_addr.send(start).await;

    let game_state = server_addr
        .send(GetGameState("lobby".into()))
        .await
        .unwrap()
        .0
        .unwrap();

    // Assert
    assert_eq!(
        game_state.current_turn,
        TeamSymbol::Cross,
        "The current turn does not change."
    );
    assert_eq!(
        game_state.status,
        GameRoomStatus::Waiting,
        "Does not start the game."
    );
}

#[actix_web::test]
async fn circle_player_cant_start_game() {
    // Arrange
    let server = setup_game_server_with_status(GameRoomStatus::Waiting);
    let player_id = get_player_ids_from_room(&server, "lobby")[1];
    let server_addr = server.start();

    let start = StartGame {
        id: player_id,
        team_symbol: Some(TeamSymbol::Circle),
    };

    // Act
    let _ = server_addr.send(start).await;

    let game_state = server_addr
        .send(GetGameState("lobby".into()))
        .await
        .unwrap()
        .0
        .unwrap();

    // Assert
    assert_eq!(
        game_state.current_turn,
        TeamSymbol::Cross,
        "The current turn does not change."
    );
    assert_eq!(
        game_state.status,
        GameRoomStatus::Waiting,
        "Does not start the game."
    );
}

#[actix_web::test]
async fn cross_player_can_start_game() {
    // Arrange
    let server = setup_game_server_with_status(GameRoomStatus::Waiting);
    let player_id = get_player_ids_from_room(&server, "lobby")[0];
    let server_addr = server.start();

    let start = StartGame {
        id: player_id,
        team_symbol: Some(TeamSymbol::Cross),
    };

    // Act
    let _ = server_addr.send(start).await;

    let game_state = server_addr
        .send(GetGameState("lobby".into()))
        .await
        .unwrap()
        .0
        .unwrap();

    // Assert
    assert_eq!(
        game_state.current_turn,
        TeamSymbol::Cross,
        "The current turn does not change."
    );
    assert_eq!(
        game_state.status,
        GameRoomStatus::Started,
        "Starts the game."
    );
}
