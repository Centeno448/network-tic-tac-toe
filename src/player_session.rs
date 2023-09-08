use actix::prelude::*;
use actix_web_actors::ws;
use uuid::Uuid;

use crate::game_server;

/// Define HTTP actor
pub struct PlayerSession {
    pub id: Uuid,
    pub team_symbol: Option<game_server::domain::TeamSymbol>,
    pub username: String,
    pub game_server_addr: Addr<game_server::GameServer>,
}

impl Actor for PlayerSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let session_addr = ctx.address();
        self.game_server_addr
            .send(game_server::events::Connect {
                id: self.id,
                addr: session_addr.recipient(),
            })
            .into_actor(self)
            .then(|res, _, ctx| {
                match res {
                    Ok(_) => {}
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> actix::Running {
        self.game_server_addr
            .do_send(game_server::events::Disconnect { id: self.id });

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
                        .send(game_server::events::StartGame {
                            id: self.id,
                            team_symbol: self.team_symbol,
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
                } else if trimmed_text.starts_with("/create") {
                    let room_name: Vec<&str> = trimmed_text.split(' ').collect();

                    if let Some(room_name) = room_name.get(1) {
                        let room_name = *room_name;
                        self.game_server_addr
                            .send(game_server::events::CreateMatch {
                                id: self.id,
                                room_name: room_name.into(),
                                username: self.username.clone(),
                            })
                            .into_actor(self)
                            .then(|res, session, ctx| {
                                match res {
                                    Ok(_) => {
                                        session.team_symbol =
                                            Some(game_server::domain::TeamSymbol::Cross);
                                    }
                                    _ => ctx.stop(),
                                }
                                fut::ready(())
                            })
                            .wait(ctx);
                    }
                } else if trimmed_text.starts_with("/join") {
                    let room_id: Vec<&str> = trimmed_text.split(' ').collect();

                    if let Some(room_id) = room_id.get(1) {
                        if let Ok(room_id) = Uuid::try_parse(room_id) {
                            self.game_server_addr
                                .send(game_server::events::JoinMatch {
                                    player_id: self.id,
                                    room_id,
                                    username: self.username.clone(),
                                })
                                .into_actor(self)
                                .then(|res, session, ctx| {
                                    match res {
                                        Ok(_) => {
                                            session.team_symbol =
                                                Some(game_server::domain::TeamSymbol::Circle);
                                        }
                                        _ => ctx.stop(),
                                    }
                                    fut::ready(())
                                })
                                .wait(ctx);
                        }
                    }
                } else if trimmed_text.starts_with("/list") {
                    self.game_server_addr
                        .send(game_server::events::ListMatches { player_id: self.id })
                        .into_actor(self)
                        .then(|res, _, ctx| {
                            match res {
                                Ok(_) => (),
                                _ => ctx.stop(),
                            }
                            fut::ready(())
                        })
                        .wait(ctx);
                } else if trimmed_text.starts_with("/username") {
                    let user_name: Vec<&str> = trimmed_text.split(' ').collect();

                    if let Some(user_name) = user_name.get(1) {
                        let user_name = *user_name;
                        self.username = user_name.into();
                    }
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

impl Handler<game_server::ServerMessage> for PlayerSession {
    type Result = ();
    fn handle(&mut self, msg: game_server::ServerMessage, ctx: &mut Self::Context) -> Self::Result {
        if msg.0.contains("PlayerDisconnected")
            && self.team_symbol == Some(game_server::domain::TeamSymbol::Circle)
        {
            self.team_symbol = Some(game_server::domain::TeamSymbol::Cross);
        }

        ctx.text(msg.0);
    }
}
