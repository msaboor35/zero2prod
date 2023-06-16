#[derive(serde::Deserialize, Debug)]
pub struct Settings {
    pub db: DatabaseSettings,
    pub port: u16,
}

#[derive(serde::Deserialize, Debug)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub name: String,
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let config = config::Config::builder()
        .add_source(config::File::with_name("settings"))
        .build()
        .unwrap();

    config.try_deserialize::<Settings>()
}
