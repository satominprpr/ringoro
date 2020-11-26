use mongodm::{
    mongo::{options::ClientOptions, Client, Database},
    Model, Repository, ToRepository,
};

use crate::config::Config;
use crate::result::Result;

pub struct Context {
    client: Client,
    database_name: String,
}

impl Context {
    pub async fn new(config: &Config) -> Result<Self> {
        let option = ClientOptions::parse(&config.db_uri).await?;
        let client = Client::with_options(option)?;
        Ok(Context {
            client,
            database_name: config.db_database.clone(),
        })
    }

    #[cfg(test)]
    pub async fn build_in_test(uri: &str, database_name: &str) -> Result<Self> {
        let option = ClientOptions::parse(uri).await?;
        let client = Client::with_options(option)?;
        Ok(Context {
            client,
            database_name: String::from(database_name),
        })
    }

    pub fn database(&self) -> Database {
        self.client.database(&self.database_name)
    }

    pub fn repo<M>(&self) -> Repository<M>
    where
        M: Model,
    {
        self.database().repository::<M>()
    }
}

impl juniper::Context for Context {}
