use actix::Actor;
use uuid::Uuid;

use network_tic_tac_toe::game_server::domain::TeamSymbol;
use network_tic_tac_toe::game_server::events::{Connect, GetGameState};
use network_tic_tac_toe::game_server::GameRoomStatus;

use crate::helpers::{setup_empty_game_server, MockPlayerSession};

#[actix_web::test]
async fn first_player_to_connect_is_assigned_cross() {
    // Arrange
    let server = setup_empty_game_server();
    let server_addr = server.start();

    let mock_addr = MockPlayerSession::new().start();

    let connect = Connect {
        id: Uuid::new_v4(),
        addr: mock_addr.recipient(),
    };

    // Act
    let res = server_addr.send(connect).await;

    let assigned_symbol = res.unwrap();
    let game_state = server_addr
        .send(GetGameState("lobby".into()))
        .await
        .unwrap()
        .0
        .unwrap();

    // Assert
    assert_eq!(
        assigned_symbol,
        TeamSymbol::Cross,
        "Player is assigned cross"
    );
    assert_eq!(
        game_state.current_turn,
        TeamSymbol::Cross,
        "The current turn does not change."
    );
    assert_eq!(
        game_state.status,
        GameRoomStatus::Waiting,
        "Does not start the game"
    );
}

#[actix_web::test]
async fn second_player_to_connect_is_assigned_circle() {
    // Arrange
    let server = setup_empty_game_server();
    let server_addr = server.start();

    let mock_addr = MockPlayerSession::new().start();
    let mock_addr2 = MockPlayerSession::new().start();

    let connect = Connect {
        id: Uuid::new_v4(),
        addr: mock_addr.recipient(),
    };

    let connect2 = Connect {
        id: Uuid::new_v4(),
        addr: mock_addr2.recipient(),
    };

    // Act
    let _ = server_addr.send(connect).await;
    let res = server_addr.send(connect2).await;

    let assigned_symbol = res.unwrap();
    let game_state = server_addr
        .send(GetGameState("lobby".into()))
        .await
        .unwrap()
        .0
        .unwrap();

    // Assert
    assert_eq!(
        assigned_symbol,
        TeamSymbol::Circle,
        "Player is assigned circle"
    );
    assert_eq!(
        game_state.current_turn,
        TeamSymbol::Cross,
        "The current turn does not change."
    );
    assert_eq!(
        game_state.status,
        GameRoomStatus::Waiting,
        "Does not start the game"
    );
}
