use actix::prelude::*;
use uuid::Uuid;

use crate::game_server::{GameServer, GameState};

#[derive(Message, Debug)]
#[rtype(result = "GameState")]
pub struct GetGameState(pub String);

impl Handler<GetGameState> for GameServer {
    type Result = GameState;

    #[tracing::instrument(
        name = "Get Game State",
        skip_all,
        fields(room_name=%msg.0)
    )]
    fn handle(&mut self, msg: GetGameState, _: &mut Self::Context) -> Self::Result {
        match self.rooms.get(&Uuid::new_v4()) {
            None => {
                tracing::info!("Room {} not found", &msg.0);
                GameState(None)
            }
            Some(room) => GameState(Some(room.clone())),
        }
    }
}
