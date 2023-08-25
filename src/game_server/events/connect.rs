use actix::prelude::*;
use std::sync::atomic::Ordering;
use uuid::Uuid;

use crate::game_server::{CommandCategory, Commmand, GameServer, ServerMessage};

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub id: Uuid,
    pub addr: Recipient<ServerMessage>,
}

impl Handler<Connect> for GameServer {
    type Result = ();

    #[tracing::instrument(name = "Player connect", skip_all, fields(player_session_id=%msg.id))]
    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        let id = msg.id;

        let _ = self.visitor_count.fetch_add(1, Ordering::SeqCst);
        let count = self.visitor_count.load(Ordering::Relaxed);
        tracing::info!("Number of players connected: {count}");

        let connect_command = Commmand::new_serialized(CommandCategory::Connected, "");
        self.send_direct_message(&msg.addr, &connect_command);

        self.sessions.insert(id, msg.addr);
    }
}
