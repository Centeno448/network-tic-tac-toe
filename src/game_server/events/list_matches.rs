use actix::prelude::*;
use uuid::Uuid;

use crate::game_server::{CommandCategory, Commmand, GameServer};

#[derive(Message)]
#[rtype(result = "()")]
pub struct ListMatches {
    pub player_id: Uuid,
}

impl Handler<ListMatches> for GameServer {
    type Result = ();

    #[tracing::instrument(name = "List matches", skip_all, fields(player_session_id=%msg.player_id))]
    fn handle(&mut self, msg: ListMatches, _: &mut Context<Self>) -> Self::Result {
        if let Some(addr) = self.sessions.get(&msg.player_id) {
            let mut results: Vec<serde_json::Value> = vec![];

            for (room_id, room) in self.rooms.iter() {
                let value = serde_json::json!({
                    "match_id": room_id.to_owned(),
                    "room_name": room.name.to_owned(),
                    "status": room.status.to_owned(),
                });

                results.push(value);
            }

            let command = Commmand::new_serialized(
                CommandCategory::MatchList,
                serde_json::json!({ "matches": results }),
            );

            self.send_direct_message(addr, &command);
        } else {
            tracing::info!("User not found in server sessions");
        }
    }
}
