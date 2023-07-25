use config::{self, Config};
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct AppConfig {
    pub database_url: String,
    // the port on which the http server should listen on
    pub server_port: i64,
}

pub fn create_config() -> Result<AppConfig, Box<dyn std::error::Error>> {
    let config = Config::builder()
        .add_source(config::Environment::default())
        .build()
        .unwrap();

    let app_config: AppConfig = config.try_deserialize()?;
    Ok(app_config)
}
