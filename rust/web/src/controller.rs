use std::sync::Arc;

use actix_service::ServiceFactory;
use actix_session::Session;
use actix_web::{
    dev::{MessageBody, ServiceRequest, ServiceResponse},
    get,
    http::header,
    post, web, App, HttpResponse,
};
use futures::TryStreamExt;
use serde::Deserialize;

use crate::{
    app_data::AppData, app_error::Responce, auth, context::Context, services::*,
    utils::result::Result,
};

macro_rules! entry {
    ([$path:tt] async fn $name:ident ($input:ty) = $service:path) => {
        #[post($path)]
        #[allow(clippy::unit_arg)]
        async fn $name(
            session: Session,
            query: web::Json<$input>,
            st: web::Data<Arc<AppData>>,
        ) -> Responce {
            let ctx = ctx(session, st).await?;
            let result = $service(query.into_inner(), &ctx).await?;
            Ok(HttpResponse::Ok().json(result))
        }
    };
}

macro_rules! list_entry {
    ([$path:tt] async fn $name:ident ($input:ty) -> [$output:ty] = $service:path) => {
        #[post($path)]
        #[allow(clippy::unit_arg)]
        async fn $name(
            session: Session,
            query: web::Json<$input>,
            st: web::Data<Arc<AppData>>,
        ) -> Responce {
            let ctx = ctx(session, st).await?;
            let result = $service(query.into_inner(), &ctx)
                .await?
                .try_collect::<Vec<$output>>()
                .await?;
            Ok(HttpResponse::Ok().json(result))
        }
    };
}

pub fn route<V, T>(app: App<T, V>) -> App<T, V>
where
    V: MessageBody,
    T: ServiceFactory<
        Config = (),
        Request = ServiceRequest,
        Response = ServiceResponse<V>,
        Error = actix_web::Error,
        InitError = (),
    >,
{
    app.service(login)
        .service(twiter_callback)
        .service(test_create_user)
        .service(get_user)
        .service(get_users)
        .service(delete_user)
}

#[get("/login")]
async fn login(session: Session, st: web::Data<Arc<AppData>>) -> Responce {
    let redirect = auth::twitter_login_request(&st.config, &session).await?;
    Ok(HttpResponse::Found()
        .set_header(header::LOCATION, redirect)
        .finish())
}

#[derive(Deserialize)]
pub struct TwitterCallbackRequest {
    pub oauth_verifier: String,
}

#[get("/twiter_callback")]
async fn twiter_callback(
    session: Session,
    query: web::Query<TwitterCallbackRequest>,
    st: web::Data<Arc<AppData>>,
) -> Responce {
    let verifier = query.oauth_verifier.clone();
    let context = Context::new(st.context.clone(), None);
    auth::login(&verifier, &st.config, &session, &context).await?;
    Ok(HttpResponse::Found()
        .set_header(header::LOCATION, "/")
        .finish())
}

#[cfg(debug_assertions)]
#[derive(Deserialize)]
pub struct TestCreateUser {
    pub username: String,
    pub admin: bool,
}

#[cfg(debug_assertions)]
#[get("/test_create_user")]
async fn test_create_user(
    session: Session,
    query: web::Query<TestCreateUser>,
    st: web::Data<Arc<AppData>>,
) -> Responce {
    let context = Context::new(st.context.clone(), None);
    auth::test_create_user(&query.username, query.admin, &session, &context).await?;
    Ok(HttpResponse::Ok().finish())
}

#[cfg(not(debug_assertions))]
#[get("/test_create_user")]
async fn test_create_user(
    session: Session,
    query: web::Query<TestCreateUser>,
    st: web::Data<Arc<AppData>>,
) -> Responce {
    panic!()
}

entry! {
    ["/api/user"]
    async fn get_user(UserFindOneInput) = UserService::find_one
}

list_entry! {
    ["/api/users"]
    async fn get_users(()) -> [UserOutput] = UserService::find_many
}

entry! {
    ["/api/delete_user"]
    async fn delete_user(UserFindOneInput) = UserService::delete
}

async fn ctx(session: Session, st: web::Data<Arc<AppData>>) -> Result<Context> {
    let dummy = Context::new(st.context.clone(), None);
    let user = auth::check_login(&st.config, &session, &dummy).await?;
    Ok(Context::new(st.context.clone(), user))
}
