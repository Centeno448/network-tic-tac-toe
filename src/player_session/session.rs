use actix::prelude::*;
use actix_web_actors::ws;
use uuid::Uuid;

use crate::game_server;
use crate::player_session::PlayerMessage;

/// Define HTTP actor
pub struct PlayerSession {
    pub id: Uuid,
    pub team_symbol: Option<game_server::domain::TeamSymbol>,
    pub username: String,
    pub room_id: Option<Uuid>,
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
                match serde_json::from_str::<PlayerMessage>(trimmed_text) {
                    Ok(message) => match message {
                        PlayerMessage::Start => {
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
                        }
                        PlayerMessage::Turn(turn) => {
                            let turn = turn.as_str();
                            self.game_server_addr
                                .send(game_server::events::Turn {
                                    id: self.id,
                                    team_symbol: self.team_symbol,
                                    turn_move: turn.into(),
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
                        PlayerMessage::Create(room_name) => {
                            let room_name = room_name.as_str();
                            self.game_server_addr
                                .send(game_server::events::CreateMatch {
                                    id: self.id,
                                    room_name: room_name.into(),
                                    username: self.username.clone(),
                                })
                                .into_actor(self)
                                .then(|res, session, ctx| {
                                    match res {
                                        Ok(room_id) => {
                                            session.team_symbol =
                                                Some(game_server::domain::TeamSymbol::Cross);
                                            session.room_id = room_id.0;
                                        }
                                        _ => ctx.stop(),
                                    }
                                    fut::ready(())
                                })
                                .wait(ctx);
                        }
                        PlayerMessage::Join(room_id) => {
                            self.game_server_addr
                                .send(game_server::events::JoinMatch {
                                    player_id: self.id,
                                    room_id,
                                    username: self.username.clone(),
                                })
                                .into_actor(self)
                                .then(|res, session, ctx| {
                                    match res {
                                        Ok(room_id) => {
                                            session.team_symbol =
                                                Some(game_server::domain::TeamSymbol::Circle);
                                            session.room_id = room_id.0;
                                        }
                                        _ => ctx.stop(),
                                    }
                                    fut::ready(())
                                })
                                .wait(ctx);
                        }
                        PlayerMessage::List => {
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
                        }
                        PlayerMessage::Username(username) => {
                            let username = username.chars().take(30).collect();
                            let _ = tracing::info_span!(
                                "Set username",
                                player_session_id = self.id.to_string(),
                                username = username
                            )
                            .enter();
                            self.username = username;
                        }
                    },
                    Err(_) => {
                        tracing::info!("Invalid message {}", trimmed_text);
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
