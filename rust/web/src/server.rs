use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{middleware, web, App, Error, HttpResponse, HttpServer};
use juniper::http::GraphQLRequest;

use crate::config::Config;
use crate::mongo::Context;
use crate::schema::{create_schema, Schema};

struct GraphqlAppData {
    schema: Schema,
    context: Context,
}

async fn graphql(
    st: web::Data<Arc<GraphqlAppData>>,
    data: web::Json<GraphQLRequest>,
) -> Result<HttpResponse, Error> {
    let user = web::block(move || {
        let res = data.execute(&st.schema, &st.context);
        Ok::<_, serde_json::error::Error>(serde_json::to_string(&res)?)
    })
    .await?;
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(user))
}

pub async fn run() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=trace");
    env_logger::init();

    let config = Config::from_env()?;
    let context = Context::new(&config).await?;
    let schema = create_schema();
    let data = Arc::new(GraphqlAppData { schema, context });

    HttpServer::new(move || {
        App::new()
            .data(Arc::clone(&data))
            .wrap(middleware::Logger::default())
            .wrap(
                Cors::new()
                    .allowed_methods(vec!["POST", "GET"])
                    .supports_credentials()
                    .max_age(3600)
                    .finish(),
            )
            .service(web::resource("/graphql").route(web::post().to(graphql)))
    })
    .bind(config.bind_name())?
    .run()
    .await
}
