use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use tracing_actix_web::TracingLogger;

use crate::configuration::ApplicationSettings;
use crate::player_session::index;

pub struct Application {
    port: u16,
    server: Server,
}

pub struct ApplicationBaseUrl(pub String);

impl Application {
    pub async fn build(configuration: ApplicationSettings) -> Result<Self, anyhow::Error> {
        let port = configuration.port;
        let server = run(
            configuration.host,
            configuration.port,
            configuration.base_url,
        )
        .await?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub async fn run(host: String, port: u16, base_url: String) -> Result<Server, anyhow::Error> {
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/", web::get().to(index))
            .app_data(base_url.clone())
    })
    .bind((host, port))?
    .run();

    Ok(server)
}
