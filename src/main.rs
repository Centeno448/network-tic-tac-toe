use actix_web::{web, App, HttpServer};

use network_tic_tac_toe::index;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/", web::get().to(index)))
        .bind(("127.0.0.1", 3012))?
        .run()
        .await
}
