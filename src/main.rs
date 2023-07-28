use dotenv::dotenv;
use std::fs::File;
use std::io::ErrorKind;
use std::sync::Mutex;

use network_tic_tac_toe::configuration::get_configuration;
use network_tic_tac_toe::startup::Application;
use network_tic_tac_toe::telemetry::{get_subscriber, init_subscriber};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let log_file = File::create("network-tic-tac-toe.log")?;
    let subscriber = get_subscriber(
        "network-tic-tac-toe".into(),
        "info".into(),
        Mutex::new(log_file),
    );
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let application = Application::build(configuration.clone())
        .await
        .map_err(|e| std::io::Error::new(ErrorKind::Interrupted, e))?;

    application.run_until_stopped().await
}
