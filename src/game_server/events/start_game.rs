use actix::prelude::*;
use uuid::Uuid;

use crate::game_server::{
    domain::TeamSymbol, CommandCategory, Commmand, GameRoom, GameRoomStatus, GameServer,
};

#[derive(Message)]
#[rtype(result = "()")]
pub struct StartGame {
    pub player_id: Uuid,
    pub room_id: Option<Uuid>,
    pub team_symbol: Option<TeamSymbol>,
}

impl StartGame {
    fn team_symbol_to_string(&self) -> String {
        self.team_symbol
            .map(|ts| ts.to_string())
            .unwrap_or("".into())
    }
}

impl Handler<StartGame> for GameServer {
    type Result = ();

    #[tracing::instrument(name = "Game Start", skip_all, fields(player_id=%msg.player_id, team_symbol=%msg.team_symbol_to_string(), room_id))]
    fn handle(&mut self, msg: StartGame, _: &mut Self::Context) -> Self::Result {
        if let Some(room_id) = &msg.room_id {
            tracing::Span::current().record("room_id", &room_id.to_string());

            if let Some(room) = find_waiting_game_room(self, room_id) {
                if msg.team_symbol != Some(TeamSymbol::Cross) {
                    tracing::info!("Circle player attempted to start the game, ignoring.");
                    return;
                }

                room.status = GameRoomStatus::Started;

                let command = Commmand::new_serialized(CommandCategory::GameStart, "");
                self.send_message_all(room_id, &command);
            } else {
                tracing::info!("Player is not in any room with 2 players and status waiting.");
            }
        }
    }
}

fn find_waiting_game_room<'a>(
    server: &'a mut GameServer,
    room_id: &'a Uuid,
) -> Option<&'a mut GameRoom> {
    server
        .rooms
        .get_mut(room_id)
        .filter(|r| r.status == GameRoomStatus::Waiting && r.players.iter().count() == 2)
}
