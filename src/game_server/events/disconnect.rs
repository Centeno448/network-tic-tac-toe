use actix::prelude::*;
use std::sync::atomic::Ordering;
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
        if self.sessions.remove(&msg.id).is_some() {
            for (_, room) in &mut self.rooms {
                room.players.remove(&msg.id);
            }
        }

        let _ = self.visitor_count.fetch_sub(1, Ordering::SeqCst);
        let count = self.visitor_count.load(Ordering::Relaxed);
        tracing::info!("Number of players in lobby: {count}");

        let command = Commmand::new(CommandCategory::PlayerDisconnected, msg.id.to_string());

        let result = serde_json::to_string(&command).unwrap_or("".into());

        self.send_message("lobby", &result, msg.id);
    }
}
