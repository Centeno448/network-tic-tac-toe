use serde_aux::field_attributes::deserialize_number_from_string;

#[derive(serde::Deserialize, Clone)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub log_location: String,
}

pub fn get_configuration() -> Result<ApplicationSettings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory.");
    let configuration_path = base_path.join("configuration");

    let environment: Environment = std::env::var("NTTT__ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse NTTT__ENVIRONMENT");

    let builder = config::Config::builder()
        .add_source(config::File::from(configuration_path.join("base")).required(true))
        .add_source(
            config::File::from(configuration_path.join(environment.as_str())).required(true),
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
