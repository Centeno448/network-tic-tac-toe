use actix::Actor;

use network_tic_tac_toe::game_server::domain::{TeamSymbol, TurnMove};
use network_tic_tac_toe::game_server::events::{GetGameState, Turn};
use network_tic_tac_toe::game_server::GameRoomStatus;
use network_tic_tac_toe::player_session::PlayerSession;

use crate::helpers::{
    get_player_ids_from_room, setup_game_for_circle_victory, setup_game_for_cross_victory,
    setup_game_for_diagonal_victory, setup_game_for_tie, setup_game_server_with_status,
};

#[actix_web::test]
async fn server_ignores_invalid_turn() {
    // Arrange
    let server = setup_game_server_with_status(GameRoomStatus::Started);
    let player_ids = get_player_ids_from_room(&server, "lobby");
    let server_addr = server.start();
    let player_session = PlayerSession {
        id: player_ids[0],
        team_symbol: Some(TeamSymbol::Circle),
        game_server_addr: server_addr,
    };
    let turn = Turn {
        id: player_session.id,
        team_symbol: player_session.team_symbol,
        turn_move: TurnMove::MM,
    };

    // Act
    let _ = player_session.game_server_addr.send(turn).await;

    let game_state = player_session
        .game_server_addr
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
    assert!(
        !game_state.moves_made.contains_key(&TurnMove::MM),
        "The invalid turn is not stored."
    );
    assert_eq!(
        game_state.status,
        GameRoomStatus::Started,
        "Does not end the game."
    );
}

#[actix_web::test]
async fn server_ignores_duplicate_turn() {
    // Arrange
    let mut server = setup_game_server_with_status(GameRoomStatus::Started);
    let player_ids = get_player_ids_from_room(&server, "lobby");
    server
        .rooms
        .get_mut("lobby")
        .unwrap()
        .moves_made
        .insert(TurnMove::MM, player_ids[0].clone());
    let server_addr = server.start();
    let player_session = PlayerSession {
        id: player_ids[0],
        team_symbol: Some(TeamSymbol::Cross),
        game_server_addr: server_addr,
    };
    let turn = Turn {
        id: player_session.id,
        team_symbol: player_session.team_symbol,
        turn_move: TurnMove::MM,
    };

    // Act
    let _ = player_session.game_server_addr.send(turn).await;

    let game_state = player_session
        .game_server_addr
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
        "Does not end the game."
    );
}

#[actix_web::test]
async fn server_ignores_invalid_player() {
    // Arrange
    let server = setup_game_server_with_status(GameRoomStatus::Started);
    let player_ids = get_player_ids_from_room(&server, "lobby");
    let server_addr = server.start();
    let player_session = PlayerSession {
        id: player_ids[0],
        team_symbol: Some(TeamSymbol::Cross),
        game_server_addr: server_addr,
    };
    let turn = Turn {
        id: uuid::Uuid::new_v4(),
        team_symbol: player_session.team_symbol,
        turn_move: TurnMove::MM,
    };

    // Act
    let _ = player_session.game_server_addr.send(turn).await;

    let game_state = player_session
        .game_server_addr
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
    assert!(
        !game_state.moves_made.contains_key(&TurnMove::MM),
        "The move from the invalid player is not stored."
    );
    assert_eq!(
        game_state.status,
        GameRoomStatus::Started,
        "Does not end the game."
    );
}

#[actix_web::test]
async fn server_processes_valid_turn() {
    // Arrange
    let server = setup_game_server_with_status(GameRoomStatus::Started);
    let player_ids = get_player_ids_from_room(&server, "lobby");
    let server_addr = server.start();
    let player_session = PlayerSession {
        id: player_ids[0],
        team_symbol: Some(TeamSymbol::Cross),
        game_server_addr: server_addr,
    };
    let turn = Turn {
        id: player_session.id,
        team_symbol: player_session.team_symbol,
        turn_move: TurnMove::MM,
    };

    // Act
    let _ = player_session.game_server_addr.send(turn).await;

    let game_state = player_session
        .game_server_addr
        .send(GetGameState("lobby".into()))
        .await
        .unwrap()
        .0
        .unwrap();

    // Assert
    assert_eq!(
        game_state.current_turn,
        TeamSymbol::Circle,
        "The current turn changes."
    );
    assert!(
        game_state.moves_made.contains_key(&TurnMove::MM),
        "The move is stored."
    );
    assert_eq!(
        game_state.status,
        GameRoomStatus::Started,
        "Does not end the game."
    );
}

#[actix_web::test]
async fn game_ends_on_tie() {
    // Arrange
    let mut server = setup_game_server_with_status(GameRoomStatus::Started);
    let player_ids = get_player_ids_from_room(&server, "lobby");
    setup_game_for_tie(&mut server, player_ids[0], player_ids[1]);
    let server_addr = server.start();
    let player_session = PlayerSession {
        id: player_ids[0],
        team_symbol: Some(TeamSymbol::Cross),
        game_server_addr: server_addr,
    };
    let turn = Turn {
        id: player_session.id,
        team_symbol: player_session.team_symbol,
        turn_move: TurnMove::ML,
    };

    // Act
    let _ = player_session.game_server_addr.send(turn).await;

    let game_state = player_session
        .game_server_addr
        .send(GetGameState("lobby".into()))
        .await
        .unwrap()
        .0
        .unwrap();

    // Assert
    assert!(
        game_state.moves_made.contains_key(&TurnMove::ML),
        "The move is stored."
    );
    assert_eq!(
        game_state.status,
        GameRoomStatus::Finished,
        "Ends the game."
    );
}

#[actix_web::test]
async fn game_ends_on_diagonal_victory() {
    // Arrange
    let mut server = setup_game_server_with_status(GameRoomStatus::Started);
    let player_ids = get_player_ids_from_room(&server, "lobby");
    setup_game_for_diagonal_victory(&mut server, &player_ids[0], &player_ids[1]);
    let server_addr = server.start();
    let player_session = PlayerSession {
        id: player_ids[0],
        team_symbol: Some(TeamSymbol::Cross),
        game_server_addr: server_addr,
    };
    let turn = Turn {
        id: player_session.id,
        team_symbol: player_session.team_symbol,
        turn_move: TurnMove::UR,
    };

    // Act
    let _ = player_session.game_server_addr.send(turn).await;

    let game_state = player_session
        .game_server_addr
        .send(GetGameState("lobby".into()))
        .await
        .unwrap()
        .0
        .unwrap();

    // Assert
    assert!(
        game_state.moves_made.contains_key(&TurnMove::UR),
        "The move is stored."
    );
    assert_eq!(
        game_state.status,
        GameRoomStatus::Finished,
        "Ends the game."
    );
}

#[actix_web::test]
async fn game_ends_on_cross_victory() {
    // Arrange
    let mut server = setup_game_server_with_status(GameRoomStatus::Started);
    let player_ids = get_player_ids_from_room(&server, "lobby");
    setup_game_for_cross_victory(&mut server, &player_ids[0], &player_ids[1]);
    let server_addr = server.start();
    let player_session = PlayerSession {
        id: player_ids[0],
        team_symbol: Some(TeamSymbol::Cross),
        game_server_addr: server_addr,
    };
    let turn = Turn {
        id: player_session.id,
        team_symbol: player_session.team_symbol,
        turn_move: TurnMove::LR,
    };

    // Act
    let _ = player_session.game_server_addr.send(turn).await;

    let game_state = player_session
        .game_server_addr
        .send(GetGameState("lobby".into()))
        .await
        .unwrap()
        .0
        .unwrap();

    // Assert
    assert!(
        game_state.moves_made.contains_key(&TurnMove::LR),
        "The move is stored."
    );
    assert_eq!(
        game_state.status,
        GameRoomStatus::Finished,
        "Ends the game."
    );
}

#[actix_web::test]
async fn game_ends_on_circle_victory() {
    // Arrange
    let mut server = setup_game_server_with_status(GameRoomStatus::Started);
    let player_ids = get_player_ids_from_room(&server, "lobby");
    setup_game_for_circle_victory(&mut server, &player_ids[0], &player_ids[1]);
    let server_addr = server.start();
    let player_session = PlayerSession {
        id: player_ids[1],
        team_symbol: Some(TeamSymbol::Circle),
        game_server_addr: server_addr,
    };
    let turn = Turn {
        id: player_session.id,
        team_symbol: player_session.team_symbol,
        turn_move: TurnMove::UR,
    };

    // Act
    let _ = player_session.game_server_addr.send(turn).await;

    let game_state = player_session
        .game_server_addr
        .send(GetGameState("lobby".into()))
        .await
        .unwrap()
        .0
        .unwrap();

    // Assert
    assert!(
        game_state.moves_made.contains_key(&TurnMove::UR),
        "The move is stored."
    );
    assert_eq!(
        game_state.status,
        GameRoomStatus::Finished,
        "Ends the game."
    );
}
