use actix::prelude::*;
use actix_web_actors::ws;
use uuid::Uuid;

use crate::game_server;

/// Define HTTP actor
pub struct PlayerSession {
    pub id: Uuid,
    pub team_symbol: Option<game_server::domain::TeamSymbol>,
    pub game_server_addr: Addr<game_server::GameServer>,
}

impl Actor for PlayerSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let session_addr = ctx.address();
        self.game_server_addr
            .send(game_server::Connect {
                id: self.id,
                addr: session_addr.recipient(),
            })
            .into_actor(self)
            .then(|res, player_session, ctx| {
                match res {
                    Ok(team_symbol) => {
                        player_session.team_symbol = Some(team_symbol);
                    }
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> actix::Running {
        self.game_server_addr
            .do_send(game_server::Disconnect { id: self.id });

        actix::Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for PlayerSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                let trimmed_text = text.trim();
                if trimmed_text.starts_with("/start") {
                    self.game_server_addr
                        .send(game_server::StartGame { id: self.id })
                        .into_actor(self)
                        .then(|res, _, ctx| {
                            match res {
                                Ok(_) => (),
                                _ => ctx.stop(),
                            }
                            fut::ready(())
                        })
                        .wait(ctx);
                } else if trimmed_text.starts_with("/turn") {
                    let turn_move: Vec<&str> = trimmed_text.split(' ').collect();
                    self.game_server_addr
                        .send(game_server::events::Turn {
                            id: self.id,
                            team_symbol: self.team_symbol,
                            turn_move: turn_move[1].into(),
                        })
                        .into_actor(self)
                        .then(|res, _, ctx| {
                            match res {
                                Ok(_) => (),
                                _ => ctx.stop(),
                            }
                            fut::ready(())
                        })
                        .wait(ctx);
                }
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => (),
        }
    }
}

impl Handler<game_server::Message> for PlayerSession {
    type Result = ();
    fn handle(&mut self, msg: game_server::Message, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(msg.0);
    }
}
