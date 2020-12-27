use serde::Deserialize;

use crate::result::Result;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub db_uri: String,
    pub db_database: String,
    pub redis_address: String,
    pub session_key: String,
    pub twitter_consumer_key: String,
    pub twitter_consumer_secret: String,
    pub twitter_redirect_url: String,
}

fn default_port() -> u16 {
    8080
}

impl Config {
    pub fn from_env() -> Result<Config> {
        Ok(envy::prefixed("RINGORO_").from_env::<Config>()?)
    }

    pub fn bind_name(&self) -> String {
        format!("{host}:{port}", host = self.host, port = self.port)
    }

    pub fn session_key_bin(&self) -> Result<Vec<u8>> {
        Ok(hex::decode(&self.session_key)?)
    }
}
