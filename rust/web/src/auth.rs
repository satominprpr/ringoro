use actix_session::Session;
use chrono::{DateTime, Utc};
use egg_mode::{auth, KeyPair, Token};
use log::info;
use mongodm::{doc, f};

use crate::{
    context::Context,
    mongo::withid::{RepositoryWithId, WithId},
    stores::{User, UserRepository},
    utils::{config::Config, result::Result, simple_error},
};

const CHECK_LIMIT: i64 = 600;

pub async fn check_login(
    config: &Config,
    session: &Session,
    ctx: &Context,
) -> Result<Option<WithId<User>>> {
    match session::get_last_verified_at(session) {
        Ok(last) => {
            let diff = Utc::now() - last;
            if diff.num_seconds() > CHECK_LIMIT {
                let v = verify(config, session, ctx).await;
                if v.is_err() {
                    session.clear()
                }
                v?
            }
        }
        _ => {
            session.clear();
            info!(target: "ringoro", "check login: no data");
            return Ok(None);
        }
    };
    let result = match session::get_username(session) {
        Ok(user_name) => {
            let repo = UserRepository::new(ctx).await;
            repo.find_one(doc! {f!(name in User): &user_name}).await?
        }
        _ => None,
    };
    if result.is_none() {
        info!(target: "ringoro", "check login: no data");
        session.clear();
    }
    info!(target: "ringoro", "check login: found: {:?}", result);
    Ok(result)
}

pub async fn twitter_login_request(cfg: &Config, session: &Session) -> Result<String> {
    session.renew();
    let request_token =
        auth::request_token(&build_consumer(cfg), cfg.twitter_redirect_url.clone()).await?;
    let auth_url = auth::authenticate_url(&request_token);
    session::set_request_token(session, request_token)?;
    Ok(auth_url)
}

pub async fn login(
    verifier: &impl AsRef<str>,
    cfg: &Config,
    session: &Session,
    ctx: &Context,
) -> Result<()> {
    let request_token = session::get_request_token(session)?;
    let (access_token, _, user_name) =
        auth::access_token(build_consumer(cfg), &request_token, verifier.as_ref()).await?;
    session::remove_request_token(session);
    let user = create_or_get_user(&user_name, ctx).await?;
    if let Token::Access {
        access,
        consumer: _,
    } = access_token
    {
        session::set_access_token(session, access)?;
    } else {
        return Err(simple_error!("unexpected token"));
    }
    session::set_username(session, user.1.name)?;
    session::set_last_verified_at(session)?;
    session.renew();
    Ok(())
}

#[cfg(debug_assertions)]
pub async fn test_create_user(
    user_name: impl AsRef<str>,
    admin: bool,
    session: &Session,
    ctx: &Context,
) -> Result<()> {
    if admin {
        let repo = UserRepository::new(ctx).await;
        let _ = repo
            .create(&User::new_admin_user(String::from(user_name.as_ref())))
            .await?;
    };
    let user = create_or_get_user(user_name, ctx).await?;
    session::set_username(session, user.1.name)?;
    session::set_last_verified_at(session)?;
    Ok(())
}

async fn create_or_get_user(user_name: impl AsRef<str>, ctx: &Context) -> Result<WithId<User>> {
    let repo = UserRepository::new(ctx).await;
    let user = match repo
        .find_one(doc! {f!(name in User): user_name.as_ref()})
        .await?
    {
        Some(u) => u,
        None => {
            let id = repo
                .create(&User::new(String::from(user_name.as_ref())))
                .await?;
            repo.find_one_by_id(&id)
                .await?
                .ok_or_else(|| simple_error!("invalid update"))?
        }
    };
    Ok(user)
}

async fn verify(cfg: &Config, session: &Session, ctx: &Context) -> Result<()> {
    let access_token = session::get_access_token(session)?;
    let verified_user_name = auth::verify_tokens(&build_access_token(cfg, access_token))
        .await?
        .response
        .screen_name;
    let saved_user_name = session::get_username(session)?;
    if verified_user_name == saved_user_name {
        let repo = UserRepository::new(ctx).await;
        let _ = repo
            .find_one(doc! {f!(name in User): saved_user_name})
            .await?;
        session::set_last_verified_at(session)?;
        Ok(())
    } else {
        Err(simple_error!("user name unmatch"))
    }
}

fn build_consumer(cfg: &Config) -> KeyPair {
    KeyPair::new(
        cfg.twitter_consumer_key.clone(),
        cfg.twitter_consumer_secret.clone(),
    )
}

fn build_access_token(cfg: &Config, access: KeyPair) -> Token {
    Token::Access {
        consumer: build_consumer(cfg),
        access,
    }
}

mod session {
    use super::*;
    use serde::{Deserialize, Serialize};

    const REQUEST_TOKEN_KEY: &str = "request_token";
    const ACCESS_TOKEN_KEY: &str = "access_token";
    const USERNAME_KEY: &str = "username";
    const LAST_VERIFIED_AT_KEY: &str = "last_verified_at";

    pub fn set_request_token(session: &Session, value: KeyPair) -> Result<()> {
        save(session, REQUEST_TOKEN_KEY, value)
    }

    pub fn get_request_token(session: &Session) -> Result<KeyPair> {
        load(session, REQUEST_TOKEN_KEY)
    }

    pub fn remove_request_token(session: &Session) {
        session.remove(REQUEST_TOKEN_KEY);
    }

    pub fn set_access_token(session: &Session, value: KeyPair) -> Result<()> {
        save(session, ACCESS_TOKEN_KEY, value)
    }

    pub fn get_access_token(session: &Session) -> Result<KeyPair> {
        load(session, ACCESS_TOKEN_KEY)
    }

    pub fn set_username(session: &Session, value: String) -> Result<()> {
        save(session, USERNAME_KEY, value)
    }

    pub fn get_username(session: &Session) -> Result<String> {
        load(session, USERNAME_KEY)
    }

    pub fn set_last_verified_at(session: &Session) -> Result<()> {
        save(session, LAST_VERIFIED_AT_KEY, Utc::now())
    }

    pub fn get_last_verified_at(session: &Session) -> Result<DateTime<Utc>> {
        load(session, LAST_VERIFIED_AT_KEY)
    }

    fn save<T>(session: &Session, key: &str, value: T) -> Result<()>
    where
        T: Serialize,
    {
        match session.set(key, value) {
            Ok(()) => Ok(()),
            _ => Err(simple_error!(format!("{} cannot set token", key))),
        }
    }

    fn load<T>(session: &Session, key: &str) -> Result<T>
    where
        T: for<'a> Deserialize<'a>,
    {
        match session.get::<T>(key) {
            Ok(Some(value)) => Ok(value),
            _ => Err(simple_error!(format!("{} cannot get token", key))),
        }
    }
}
