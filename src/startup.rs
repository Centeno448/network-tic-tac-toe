use actix::{Actor, Addr};
use actix_web::dev::Server;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use std::net::TcpListener;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use tracing_actix_web::TracingLogger;
use uuid::Uuid;

use crate::configuration::ApplicationSettings;
use crate::game_server::GameServer;
use crate::player_session::PlayerSession;

pub struct Application {
    port: u16,
    server: Server,
}

pub struct ApplicationBaseUrl(pub String);

impl Application {
    pub async fn build(configuration: ApplicationSettings) -> Result<Self, anyhow::Error> {
        let address = format!("{}:{}", configuration.host, configuration.port);
        let listener = TcpListener::bind(address.clone())?;

        let port = listener.local_addr().unwrap().port();

        let server = run(listener).await?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub async fn index(
    req: HttpRequest,
    stream: web::Payload,
    game_server: web::Data<Addr<GameServer>>,
) -> Result<HttpResponse, Error> {
    let player_session = PlayerSession {
        id: Uuid::new_v4(),
        team_symbol: None,
        game_server_addr: game_server.get_ref().clone(),
    };
    let resp = ws::start(player_session, &req, stream).map_err(|e| {
        tracing::error!("Error starting session {e}");
        e
    });

    resp
}

pub async fn run(listener: TcpListener) -> Result<Server, anyhow::Error> {
    let app_state = Arc::new(AtomicUsize::new(0));
    let game_server = GameServer::new(app_state.clone()).start();

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(web::Data::new(game_server.clone()))
            .route("/", web::get().to(index))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
