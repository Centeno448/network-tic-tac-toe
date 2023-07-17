use actix::{Actor, StreamHandler};
use actix_web_actors::ws;

/// Define HTTP actor
pub struct PlayerSession {
    pub player_id: usize,
}

impl Actor for PlayerSession {
    type Context = ws::WebsocketContext<Self>;
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for PlayerSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                let response = process_message(text.as_ref());

                ctx.text(response)
            }
            Ok(ws::Message::Close(reason)) => {
                println!("Closing connection");
                ctx.close(reason);
            }
            _ => (),
        }
    }
}

fn process_message(text: &str) -> String {
    match text.strip_prefix("/") {
        Some(command) => match command {
            "join" => "Ok".into(),
            _ => "".into(),
        },

        None => "".into(),
    }
}
