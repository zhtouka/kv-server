use config::{Config, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub server: ServerSettings,
}

#[derive(Debug, Deserialize)]
pub struct ServerSettings {
    pub port: u16,
    pub store: StoreSettings
}

#[derive(Debug, Deserialize)]
pub struct StoreSettings {
    pub name: String,
    pub path: Option<String>,
}

pub fn config() -> Settings {
    let path = std::env::current_dir().expect("missing current directry");
    let dir = path.join("config");

    let builder = Config::builder()
        .add_source(File::from(dir.join("config.yml")))
        .set_default("server.port", 9900).expect("failed to set default port");

    builder.build().expect("failed to build settings")
        .try_deserialize().expect("failed to deserialize settings")
}