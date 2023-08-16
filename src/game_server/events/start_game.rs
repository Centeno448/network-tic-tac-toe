use actix::prelude::*;
use uuid::Uuid;

use crate::game_server::{
    domain::TeamSymbol, CommandCategory, Commmand, GameRoom, GameRoomStatus, GameServer,
};

#[derive(Message)]
#[rtype(result = "()")]
pub struct StartGame {
    pub id: Uuid,
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

    #[tracing::instrument(name = "Game Start", skip_all, fields(room_name, player_id=%msg.id, team_symbol=%msg.team_symbol_to_string()))]
    fn handle(&mut self, msg: StartGame, _: &mut Self::Context) -> Self::Result {
        let mut result_room: Option<String> = None;

        if let Some((room_name, room)) = find_waiting_game_room(self, &msg.id) {
            tracing::Span::current().record("room_name", room_name);

            if msg.team_symbol != Some(TeamSymbol::Cross) {
                tracing::info!("Circle player attempted to start the game, ignoring.");
                return;
            }

            room.status = GameRoomStatus::Started;

            result_room = Some(room_name.clone());
        } else {
            tracing::info!("Player is not in any room with 2 players and status waiting.");
        }

        if let Some(room_name) = result_room {
            let command = Commmand::new_serialized(CommandCategory::GameStart, "");

            self.send_message_all(&room_name, &command);
        }
    }
}

fn find_waiting_game_room<'a>(
    server: &'a mut GameServer,
    id: &'a Uuid,
) -> Option<(&'a String, &'a mut GameRoom)> {
    server.rooms.iter_mut().find(|(_, room)| {
        room.status == GameRoomStatus::Waiting
            && room.players.iter().count() == 2
            && room.players.contains(id)
    })
}
