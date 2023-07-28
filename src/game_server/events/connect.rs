use actix::prelude::*;
use std::sync::atomic::Ordering;
use uuid::Uuid;

use crate::game_server::domain::TeamSymbol;
use crate::game_server::{CommandCategory, Commmand, GameRoom, GameServer, ServerMessage};

#[derive(Message)]
#[rtype(result = "TeamSymbol")]
pub struct Connect {
    pub id: Uuid,
    pub addr: Recipient<ServerMessage>,
}

impl Handler<Connect> for GameServer {
    type Result = TeamSymbol;

    #[tracing::instrument(name = "Player connect", skip_all, fields(player_session_id=%msg.id))]
    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        let id = msg.id;

        self.rooms
            .entry("lobby".to_owned())
            .or_insert_with(GameRoom::new)
            .players
            .insert(id);

        let _ = self.visitor_count.fetch_add(1, Ordering::SeqCst);
        let count = self.visitor_count.load(Ordering::Relaxed);
        tracing::info!("Number of players in lobby: {count}");

        let player_symbol = if self.rooms.get("lobby").unwrap().players.len() > 1 {
            TeamSymbol::Circle
        } else {
            TeamSymbol::Cross
        };

        let connect_command = Commmand::new_serialized(CommandCategory::Connected, &player_symbol);

        self.send_direct_message(&msg.addr, &connect_command);

        let command =
            Commmand::new_serialized(CommandCategory::PlayerConnected, msg.id.to_string());
        self.send_message("lobby", &command, id);

        self.sessions.insert(id, msg.addr);

        player_symbol
    }
}
