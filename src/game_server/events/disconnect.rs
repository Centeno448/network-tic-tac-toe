use actix::prelude::*;
use uuid::Uuid;

use crate::game_server::{
    events::utils::{handle_potential_room_deletion, remove_player_from_room},
    GameServer,
};

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub player_id: Uuid,
    pub room_id: Option<Uuid>,
}

impl Handler<Disconnect> for GameServer {
    type Result = ();

    #[tracing::instrument(
        name = "Player disconnect",
        skip_all,
        fields(player_session_id=%msg.player_id, room_id)
    )]
    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) -> Self::Result {
        if let Some(room_id) = &msg.room_id {
            tracing::Span::current().record("room_id", room_id.to_string());
            if let Some(room) = self.rooms.get_mut(room_id) {
                let should_delete_room = remove_player_from_room(room, &msg.player_id);
                handle_potential_room_deletion(should_delete_room, self, &msg.player_id, room_id);
            }
        }
        self.sessions.remove(&msg.player_id);
    }
}
