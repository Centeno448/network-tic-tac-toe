use actix::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

use crate::game_server::commands::{CommandCategory, Commmand};
use crate::game_server::domain::{TeamSymbol, TurnMove};
use crate::game_server::{GameRoom, GameRoomStatus, GameServer};

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Turn {
    pub id: Uuid,
    pub team_symbol: Option<TeamSymbol>,
    pub turn_move: TurnMove,
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
        fields(room_name, player_id=%msg.id, player_move=%msg.turn_move, player_team=%msg.team_symbol_to_string())
    )]
    fn handle(&mut self, msg: Turn, _: &mut Self::Context) -> Self::Result {
        let result_room: Option<String>;
        let is_victory;
        let is_tie;

        if let Some((room_name, room)) = find_started_room_by_player_id(self, &msg.id) {
            tracing::Span::current().record("room_name", room_name);

            if is_invalid_turn(room.current_turn, msg.team_symbol) {
                tracing::info!("Invalid turn.");
                return;
            }

            if is_duplicate_move(&msg.turn_move, &room.moves_made) {
                tracing::info!("Duplicate move.");
                return;
            }

            result_room = Some(room_name.clone());

            room.moves_made.insert(msg.turn_move.clone(), msg.id);

            is_victory = is_player_victory(&room, &msg.id, &msg.turn_move);

            is_tie = is_game_tie(&is_victory, &room);

            if !is_victory && !is_tie {
                change_turn(room);
            }
        } else {
            tracing::info!("Player is not in any room with status started.");
            return;
        }

        if let Some(room_name) = result_room {
            let command = Commmand::new_serialized(CommandCategory::Turn, msg.turn_move);

            self.send_message(&room_name, &command, msg.id);

            if is_victory {
                tracing::info!("Game ended in victory");
                let room = self.rooms.get_mut(&room_name).unwrap();
                room.status = GameRoomStatus::Finished;

                let response = serde_json::json!({
                    "outcome": "victory",
                    "winner": &room.current_turn
                });

                let command = Commmand::new_serialized(CommandCategory::GameOver, response);
                self.send_message_all(&room_name, &command);
            } else if is_tie {
                tracing::info!("Game ended in tie");
                self.rooms.get_mut(&room_name).unwrap().status = GameRoomStatus::Finished;
                let response = serde_json::json!({
                    "outcome": "tie"
                });
                let command = Commmand::new_serialized(CommandCategory::GameOver, response);
                self.send_message_all(&room_name, &command);
            }
        }
    }
}

fn find_started_room_by_player_id<'a>(
    server: &'a mut GameServer,
    id: &'a Uuid,
) -> Option<(&'a String, &'a mut GameRoom)> {
    server
        .rooms
        .iter_mut()
        .find(|(_, room)| room.players.contains(id) && room.status == GameRoomStatus::Started)
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
        .count();

    let is_column_victory = player_moves
        .iter()
        .filter(|pm| pm.to_string().ends_with(column))
        .count();

    if is_row_victory == 3 || is_column_victory == 3 {
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

fn is_game_tie(is_player_victory: &bool, game_room: &GameRoom) -> bool {
    if *is_player_victory {
        return false;
    }

    game_room.moves_made.iter().count() == 9
}
