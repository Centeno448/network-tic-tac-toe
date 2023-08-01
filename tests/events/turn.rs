use actix::Actor;

use network_tic_tac_toe::game_server::domain::{TeamSymbol, TurnMove};
use network_tic_tac_toe::game_server::events::{GetGameState, Turn};
use network_tic_tac_toe::game_server::GameRoomStatus;
use network_tic_tac_toe::player_session::PlayerSession;

use crate::helpers::{get_player_ids_from_room, setup_game_server};

#[actix_web::test]
async fn server_ignores_invalid_turn() {
    // Arrange
    let server = setup_game_server();
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
    let res = player_session.game_server_addr.send(turn).await;
    let returned_game_state = res.unwrap().0;
    let game_state = player_session
        .game_server_addr
        .send(GetGameState("lobby".into()))
        .await
        .unwrap()
        .0
        .unwrap();

    // Assert
    assert!(returned_game_state.is_none());
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
