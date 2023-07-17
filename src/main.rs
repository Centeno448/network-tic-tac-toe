use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;

use network_tic_tac_toe::player_session::PlayerSession;

pub async fn index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let player_session = PlayerSession { player_id: 1 };
    let resp = ws::start(player_session, &req, stream).map_err(|e| {
        println!("Error starting session {e}");
        e
    });

    resp
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/", web::get().to(index)))
        .bind(("127.0.0.1", 3012))?
        .run()
        .await
}
