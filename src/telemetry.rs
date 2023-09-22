use std::fs::File;
use std::sync::Mutex;
use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

struct SubscriberConfig {
    name: String,
    env_filter: String,
}

pub fn init_logger() {
    let sub_config = SubscriberConfig {
        name: "network-tic-tac-toe".into(),
        env_filter: "info".into(),
    };

    match std::env::var("NTTT__LOG_LOCATION") {
        Ok(log_location) => setup_file_subscriber(sub_config, &log_location),

        Err(_) => setup_stdout_subscriber(sub_config),
    }
}

fn setup_stdout_subscriber(sub_config: SubscriberConfig) {
    let subscriber = get_subscriber(sub_config.name, sub_config.env_filter, std::io::stdout);

    init_subscriber(subscriber);
}

fn setup_file_subscriber(sub_config: SubscriberConfig, log_location: &str) {
    let log_file_path = format!("{}/network-tic-tac-toe.log", log_location);
    let file = File::create(log_file_path).expect("Failed to create log file.");

    let subscriber = get_subscriber(sub_config.name, sub_config.env_filter, Mutex::new(file));

    init_subscriber(subscriber);
}

pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));

    let formatting_layer = BunyanFormattingLayer::new(name, sink);

    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}
