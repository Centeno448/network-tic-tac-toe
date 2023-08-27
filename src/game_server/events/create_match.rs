use actix::prelude::*;
use uuid::Uuid;

use crate::game_server::{CommandCategory, Commmand, GameRoom, GameServer};

#[derive(Message)]
#[rtype(result = "()")]
pub struct CreateMatch {
    pub id: Uuid,
    pub room_name: String,
}

impl Handler<CreateMatch> for GameServer {
    type Result = ();

    #[tracing::instrument(name = "Create match", skip_all, fields(player_session_id=%msg.id))]
    fn handle(&mut self, msg: CreateMatch, _: &mut Context<Self>) -> Self::Result {
        let room_id = Uuid::new_v4();

        self.rooms
            .insert(room_id, GameRoom::new(msg.room_name.clone()));

        self.rooms
            .get_mut(&room_id)
            .unwrap()
            .players
            .insert(msg.id.clone());

        if let Some(addr) = self.sessions.get(&msg.id) {
            let command = Commmand::new_serialized(CommandCategory::MatchCreated, msg.room_name);
            self.send_direct_message(addr, &command);
        }
    }
}
