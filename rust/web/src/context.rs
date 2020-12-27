use mongodm::{Model, Repository};

pub use crate::{
    mongo::{
        context::{Context as MongoContext, MongodmContext},
        withid::WithId,
    },
    stores::User,
};

#[derive(Clone)]
pub struct Context {
    pub mongo: MongoContext,
    pub user: Option<WithId<User>>,
}

impl Context {
    pub fn new(mongo: MongoContext, user: Option<WithId<User>>) -> Self {
        Context { mongo, user }
    }
}

impl MongodmContext for Context {
    fn repo<M>(&self) -> Repository<M>
    where
        M: Model,
    {
        self.mongo.repo()
    }
}
