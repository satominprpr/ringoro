use serde::Deserialize;

use crate::result::Result;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub db_uri: String,
    pub db_database: String,
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
}
