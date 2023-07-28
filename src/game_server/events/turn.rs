use actix::prelude::*;
use uuid::Uuid;

use crate::game_server::commands::{CommandCategory, Commmand};
use crate::game_server::domain::{TeamSymbol, TurnMove};
use crate::game_server::{GameRoomStatus, GameServer};

#[derive(Message)]
#[rtype(result = "()")]
pub struct Turn {
    pub id: Uuid,
    pub team_symbol: Option<TeamSymbol>,
    pub turn_move: TurnMove,
}

impl Handler<Turn> for GameServer {
    type Result = ();

    #[tracing::instrument(name = "Turn", skip_all, fields(room_name, player_id=%msg.id, turn_move=?msg.turn_move))]
    fn handle(&mut self, msg: Turn, _: &mut Self::Context) -> Self::Result {
        let mut result_room: Option<String> = None;

        for (room_name, room) in self
            .rooms
            .iter_mut()
            .filter(|(_, room)| room.status == GameRoomStatus::Started)
        {
            if room.players.contains(&msg.id) {
                tracing::Span::current().record("room_name", room_name);

                if !is_valid_turn(room.current_turn, msg.team_symbol) {
                    tracing::info!("Player symbol and current turn do not match.");
                    break;
                }

                result_room = Some(room_name.clone());

                room.current_turn = if room.current_turn == TeamSymbol::Circle {
                    TeamSymbol::Cross
                } else {
                    TeamSymbol::Circle
                };

                break;
            } else {
                tracing::info!("Player is not in any room with status started.");
            }
        }

        if let Some(room_name) = result_room {
            let command = Commmand::new_serialized(CommandCategory::Turn, &msg.turn_move);

            self.send_message(&room_name, &command, msg.id);
        }
    }
}

fn is_valid_turn(current_turn: TeamSymbol, player_symbol: Option<TeamSymbol>) -> bool {
    if player_symbol.is_none() {
        return false;
    }

    if current_turn != player_symbol.unwrap() {
        return false;
    }

    true
}
