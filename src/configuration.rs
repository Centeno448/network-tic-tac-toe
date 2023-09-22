use serde_aux::field_attributes::deserialize_number_from_string;
use std::path::PathBuf;

#[derive(serde::Deserialize, Clone)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
}

pub fn get_configuration() -> Result<ApplicationSettings, config::ConfigError> {
    let base_path_string = std::env::var("NTTT__CONFIG_LOCATION").unwrap_or_else(|_| ".".into());
    let mut base_path = PathBuf::new();
    base_path.push(base_path_string);
    let configuration_path = base_path.join("configuration");

    let environment: Environment = std::env::var("NTTT__ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse NTTT__ENVIRONMENT");

    let builder = config::Config::builder()
        .add_source(config::File::from(configuration_path.join("base")).required(true))
        .add_source(
            config::File::from(configuration_path.join(environment.as_str())).required(false),
        )
        .add_source(config::Environment::with_prefix("NTTT").separator("__"));

    builder.build()?.try_deserialize()
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "local" => Ok(Environment::Local),
            "production" => Ok(Environment::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either 'local' or 'production'.",
                other
            )),
        }
    }
}
