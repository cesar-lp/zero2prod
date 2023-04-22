use config::{Config, ConfigError, File, FileFormat};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Settings {
    pub application: AppSettings,
    pub database: DatabaseSettings,
}

#[derive(Deserialize)]
pub struct AppSettings {
    pub port: u16,
}

#[derive(Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn get_conection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }

    pub fn get_conection_string_without_db(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password, self.host, self.port
        )
    }
}

pub fn get_configurations() -> Result<Settings, ConfigError> {
    let settings = Config::builder()
        .add_source(File::new("configuration.yaml", FileFormat::Yaml))
        .build()?;
    settings.try_deserialize::<Settings>()
}
