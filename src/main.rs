use dotenv::dotenv;
use std::io::ErrorKind;

use network_tic_tac_toe::configuration::get_configuration;
use network_tic_tac_toe::startup::Application;
use network_tic_tac_toe::telemetry::init_logger;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    match dotenv() {
        Ok(_) => {
            println!("Loaded environment variables from env file.");
        }
        Err(e) => {
            println!("Failed to load env file with error \"{}\". Skipping.", e);
        }
    }

    init_logger();

    let configuration = get_configuration().expect("Failed to read configuration.");

    let application = Application::build(configuration.clone())
        .await
        .map_err(|e| std::io::Error::new(ErrorKind::Interrupted, e))?;

    application.run_until_stopped().await
}
