use std::sync::Arc;

use actix_cors::Cors;
use actix_redis::RedisSession;
use actix_web::{middleware, App, HttpServer};

use crate::{
    app_data::AppData,
    context::MongoContext,
    controller,
    utils::{config::Config, result::Result},
};

pub async fn run() -> Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let config = Config::from_env()?;
    let context = MongoContext::new(&config).await?;
    let redis_address = config.redis_address.clone();
    let session_key = config.session_key_bin()?;
    let bind_name = config.bind_name();
    let data = Arc::new(AppData { context, config });

    Ok(HttpServer::new(move || {
        let app = App::new()
            .data(Arc::clone(&data))
            .wrap(middleware::Logger::default())
            .wrap(
                Cors::new()
                    .allowed_methods(vec!["POST", "GET"])
                    .supports_credentials()
                    .max_age(3600)
                    .finish(),
            )
            .wrap(
                RedisSession::new(redis_address.clone(), session_key.as_ref()).cookie_secure(false),
            );
        controller::route(app)
    })
    .bind(bind_name)?
    .run()
    .await?)
}
