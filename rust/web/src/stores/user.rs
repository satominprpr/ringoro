use async_trait::async_trait;
use mongodm::{CollectionConfig, Indexes, Model};
use validator::Validate;

use crate::{
    context::Context,
    mongo::{
        context::MongodmContext,
        validate_uniqueness,
        withid::{CreateOrUpdate, ValidatedRepositoryWithId, Validator},
    },
    utils::{
        result::Result,
        serde::{Deserialize, Serialize},
    },
};

#[derive(Serialize, Deserialize, Validate, Clone, Debug)]
pub struct User {
    pub name: String,
    admin: bool,
}

impl User {
    pub fn new(name: String) -> Self {
        Self { name, admin: false }
    }

    pub fn new_admin_user(name: String) -> Self {
        Self { name, admin: true }
    }

    pub fn is_admin(&self) -> bool {
        self.admin
    }
}

pub struct UserCfg {}

impl CollectionConfig for UserCfg {
    fn collection_name() -> &'static str {
        "User"
    }

    fn indexes() -> Indexes {
        Indexes::new()
    }
}

impl Model for User {
    type CollConf = UserCfg;
}

pub struct UserValidator {}

#[async_trait(?Send)]
impl Validator for UserValidator {
    type Model = User;
    type Ctx = Context;

    async fn validate(
        cu: CreateOrUpdate,
        model: &'_ Self::Model,
        ctx: &'_ Self::Ctx,
    ) -> Result<()> {
        validate_uniqueness! (<User, name>, cu, model, ctx);
        Ok(())
    }
}

pub type UserRepository = ValidatedRepositoryWithId<User, Context, UserValidator>;
