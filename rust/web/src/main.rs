use std::io::{Error, ErrorKind, Result};

use ringoro_web::server;

#[actix_web::main]
async fn main() -> Result<()> {
    match server::run().await {
        Ok(()) => Ok(()),
        Err(err) => Err(Error::new(ErrorKind::Other, err)),
    }
}
