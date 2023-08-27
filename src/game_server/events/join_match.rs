use actix::prelude::*;
use uuid::Uuid;

use crate::game_server::{CommandCategory, Commmand, GameRoom, GameRoomStatus, GameServer};

#[derive(Message)]
#[rtype(result = "()")]
pub struct JoinMatch {
    pub player_id: Uuid,
    pub room_id: Uuid,
}

impl Handler<JoinMatch> for GameServer {
    type Result = ();

    #[tracing::instrument(name = "Join match", skip_all, fields(player_session_id=%msg.player_id))]
    fn handle(&mut self, msg: JoinMatch, _: &mut Context<Self>) -> Self::Result {
        let mut result_room: Option<(Uuid, String)> = None;
        if let Some((room_id, game_room)) = find_waiting_game_room(self, &msg.room_id) {
            game_room.players.insert(msg.player_id);
            result_room = Some((room_id.clone(), game_room.name.clone()));
        }

        if let Some((room_id, room_name)) = result_room {
            if let Some(addr) = self.sessions.get(&msg.player_id) {
                let command = Commmand::new_serialized(CommandCategory::MatchJoined, room_name);
                self.send_direct_message(addr, &command);

                let command = Commmand::new_serialized(CommandCategory::PlayerConnected, "");
                self.send_message(&room_id, &command, msg.player_id.clone());
            }
        }
    }
}

fn find_waiting_game_room<'a, 'b>(
    server: &'a mut GameServer,
    room_id: &'b Uuid,
) -> Option<(&'a Uuid, &'a mut GameRoom)> {
    server.rooms.iter_mut().find(|(id, room)| {
        room.status == GameRoomStatus::Waiting && room.players.iter().count() == 1 && *id == room_id
    })
}
