use actix::prelude::*;
use uuid::Uuid;

use crate::game_server::{
    domain::TeamSymbol, CommandCategory, Commmand, GameRoom, GameRoomStatus, GameServer,
};

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: Uuid,
}

impl Handler<Disconnect> for GameServer {
    type Result = ();

    #[tracing::instrument(
        name = "Player disconnect",
        skip_all,
        fields(player_session_id=%msg.id)
    )]
    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) -> Self::Result {
        let mut result_room: Option<Uuid> = None;
        let mut should_delete_room = false;

        if self.sessions.remove(&msg.id).is_some() {
            for (room_id, room) in &mut self.rooms {
                if room.players.contains_key(&msg.id) {
                    room.players.remove(&msg.id);
                    if room.players.len() > 0 {
                        reset_room(room);
                    } else {
                        should_delete_room = true;
                    }
                    result_room = Some(room_id.clone());
                }
            }
        }

        if let Some(room_id) = result_room {
            if should_delete_room {
                self.rooms.remove(&room_id);
            } else {
                let command = Commmand::new(CommandCategory::PlayerDisconnected, "".to_string());
                let result = serde_json::to_string(&command).unwrap_or("".into());

                self.send_message(&room_id, &result, msg.id);
            }
        }
    }
}

fn reset_room(room: &mut GameRoom) {
    room.moves_made.clear();
    room.status = GameRoomStatus::Waiting;
    room.current_turn = TeamSymbol::Cross;
}
