use crate::{context::MongoContext, utils::config::Config};

pub struct AppData {
    pub context: MongoContext,
    pub config: Config,
}
