use actix::prelude::*;
use uuid::Uuid;

use crate::game_server::{domain::RoomResponse, CommandCategory, Commmand, GameRoom, GameServer};

#[derive(Message)]
#[rtype(result = "RoomResponse")]
pub struct CreateMatch {
    pub id: Uuid,
    pub username: String,
    pub room_name: String,
}

impl Handler<CreateMatch> for GameServer {
    type Result = RoomResponse;

    #[tracing::instrument(name = "Create match", skip_all, fields(player_session_id=%msg.id))]
    fn handle(&mut self, msg: CreateMatch, _: &mut Context<Self>) -> Self::Result {
        let room_id = Uuid::new_v4();

        self.rooms
            .insert(room_id, GameRoom::new(msg.room_name.clone()));

        self.rooms
            .get_mut(&room_id)
            .unwrap()
            .players
            .insert(msg.id.clone(), msg.username);

        if let Some(addr) = self.sessions.get(&msg.id) {
            let command = Commmand::new_serialized(CommandCategory::MatchCreated, room_id);
            self.send_direct_message(addr, &command);
            return RoomResponse(Some(room_id.clone()));
        }
        return RoomResponse(None);
    }
}
