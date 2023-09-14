use actix::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

use crate::game_server::commands::{CommandCategory, Commmand};
use crate::game_server::domain::{TeamSymbol, TurnMove};
use crate::game_server::{GameRoom, GameRoomStatus, GameServer};

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Turn {
    pub player_id: Uuid,
    pub team_symbol: Option<TeamSymbol>,
    pub turn_move: TurnMove,
    pub room_id: Option<Uuid>,
}

impl Turn {
    fn team_symbol_to_string(&self) -> String {
        self.team_symbol
            .map(|ts| ts.to_string())
            .unwrap_or("".into())
    }
}

impl Handler<Turn> for GameServer {
    type Result = ();

    #[tracing::instrument(
        name = "Turn",
        skip_all,
        fields(room_id, player_id=%msg.player_id, player_move=%msg.turn_move, player_team=%msg.team_symbol_to_string(), room_id)
    )]
    fn handle(&mut self, msg: Turn, _: &mut Self::Context) -> Self::Result {
        if let Some(room_id) = &msg.room_id {
            if let Some(room) = find_started_room_by_room_id(self, room_id) {
                tracing::Span::current().record("room_id", room_id.to_string());

                if is_invalid_turn(room.current_turn, msg.team_symbol) {
                    tracing::info!("Invalid turn.");
                    return;
                }

                if is_duplicate_move(&msg.turn_move, &room.moves_made) {
                    tracing::info!("Duplicate move.");
                    return;
                }

                room.moves_made.insert(msg.turn_move.clone(), msg.player_id);

                if is_player_victory(room, &msg.player_id, &msg.turn_move) {
                    tracing::info!("Game ended in victory");
                    room.status = GameRoomStatus::Finished;
                    send_messages_victory(self, room_id, &msg);
                } else if is_game_tie(room) {
                    tracing::info!("Game ended in tie");
                    room.status = GameRoomStatus::Finished;
                    send_messages_tie(self, room_id, &msg);
                } else {
                    change_turn(room);
                    let command = Commmand::new_serialized(CommandCategory::Turn, &msg.turn_move);
                    self.send_message(room_id, &command, msg.player_id.clone());
                }
            } else {
                tracing::info!("Player is not in any room with status started.");
            }
        }
    }
}

fn send_messages_victory(server: &mut GameServer, room_id: &Uuid, msg: &Turn) {
    let command = Commmand::new_serialized(CommandCategory::Turn, &msg.turn_move);
    server.send_message(room_id, &command, msg.player_id.clone());

    if let Some(addr) = server.sessions.get(&msg.player_id) {
        let command = Commmand::new_serialized(CommandCategory::GameOver, "victory");
        server.send_direct_message(addr, &command);

        let command = Commmand::new_serialized(CommandCategory::GameOver, "defeat");
        server.send_message(room_id, &command, msg.player_id.clone());
    }
}

fn send_messages_tie(server: &mut GameServer, room_id: &Uuid, msg: &Turn) {
    let command = Commmand::new_serialized(CommandCategory::Turn, &msg.turn_move);
    server.send_message(room_id, &command, msg.player_id.clone());

    let command = Commmand::new_serialized(CommandCategory::GameOver, "tie");
    server.send_message_all(room_id, &command);
}

fn find_started_room_by_room_id<'a>(
    server: &'a mut GameServer,
    room_id: &'a Uuid,
) -> Option<&'a mut GameRoom> {
    server
        .rooms
        .get_mut(room_id)
        .filter(|r| r.status == GameRoomStatus::Started)
}

fn change_turn(room: &mut GameRoom) {
    room.current_turn = if room.current_turn == TeamSymbol::Circle {
        TeamSymbol::Cross
    } else {
        TeamSymbol::Circle
    };
}

fn is_duplicate_move(new_move: &TurnMove, moves_made: &HashMap<TurnMove, Uuid>) -> bool {
    return moves_made.get(new_move).is_some();
}

fn is_invalid_turn(current_turn: TeamSymbol, player_symbol: Option<TeamSymbol>) -> bool {
    if player_symbol.is_none() {
        return true;
    }

    if current_turn != player_symbol.unwrap() {
        return true;
    }

    false
}

fn is_player_victory(game_room: &GameRoom, player_id: &Uuid, player_move: &TurnMove) -> bool {
    let player_moves: Vec<&TurnMove> = game_room
        .moves_made
        .iter()
        .filter(|(_, v)| *v == player_id)
        .map(|(k, _)| k)
        .collect();

    let player_move = player_move.to_string();
    let mut move_iter = player_move.chars();

    let row = move_iter.next().unwrap_or(' ');
    let column = move_iter.next().unwrap_or(' ');

    let is_row_victory = player_moves
        .iter()
        .filter(|pm| pm.to_string().starts_with(row))
        .count()
        == 3;

    let is_column_victory = player_moves
        .iter()
        .filter(|pm| pm.to_string().ends_with(column))
        .count()
        == 3;

    if is_row_victory || is_column_victory {
        return true;
    }

    let is_diagonal_victory = player_moves
        .iter()
        .filter(|pm| ***pm == TurnMove::LL || ***pm == TurnMove::MM || ***pm == TurnMove::UR)
        .count()
        == 3;

    if is_diagonal_victory {
        return true;
    }

    false
}

fn is_game_tie(game_room: &GameRoom) -> bool {
    game_room.moves_made.iter().count() == 9
}
