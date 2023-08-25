use actix::prelude::*;
use uuid::Uuid;

use crate::game_server::{CommandCategory, Commmand, GameServer};

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

        if self.sessions.remove(&msg.id).is_some() {
            for (room_id, room) in &mut self.rooms {
                if room.players.contains(&msg.id) {
                    result_room = Some(room_id.clone());
                }
                room.players.remove(&msg.id);
            }
        }

        if let Some(room_id) = result_room {
            let command = Commmand::new(CommandCategory::PlayerDisconnected, msg.id.to_string());
            let result = serde_json::to_string(&command).unwrap_or("".into());

            self.send_message(&room_id, &result, msg.id);
        }
    }
}
