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

impl Handler<Turn> for GameServer {
    type Result = ();

    #[tracing::instrument(name = "Turn", skip_all, fields(room_name, msg=?msg))]
    fn handle(&mut self, msg: Turn, _: &mut Self::Context) -> Self::Result {
        let result_room: Option<String>;

        if let Some((room_name, room)) = self.rooms.iter_mut().find(|(_, room)| {
            room.players.contains(&msg.id) && room.status == GameRoomStatus::Started
        }) {
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

            change_turn(room);
        } else {
            tracing::info!("Player is not in any room with status started.");
            return;
        }

        if let Some(room_name) = result_room {
            let command = Commmand::new_serialized(CommandCategory::Turn, msg.turn_move);

            self.send_message(&room_name, &command, msg.id);
        }
    }
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
