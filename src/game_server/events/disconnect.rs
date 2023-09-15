use actix::prelude::*;
use uuid::Uuid;

use crate::game_server::{
    events::utils::{remove_player_from_room, ShouldDeleteRoom},
    CommandCategory, Commmand, GameServer,
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
                match remove_player_from_room(room, &msg.player_id) {
                    ShouldDeleteRoom::No => {
                        let command =
                            Commmand::new(CommandCategory::PlayerDisconnected, "".to_string());
                        let result = serde_json::to_string(&command).unwrap_or("".into());

                        self.send_message(room_id, &result, msg.player_id.clone());
                    }
                    ShouldDeleteRoom::Yes => {
                        self.rooms.remove(room_id);
                    }
                }
            }
        }
        self.sessions.remove(&msg.player_id);
    }
}
