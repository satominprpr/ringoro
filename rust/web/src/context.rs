pub use crate::mongo::context::Context as MongoContext;

pub struct Context {
    pub ctx: MongoContext,
}

impl Context {
    pub fn new(ctx: MongoContext) -> Self {
        Context { ctx }
    }
}

impl juniper::Context for Context {}
