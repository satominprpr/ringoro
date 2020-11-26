use std::{env::var, sync::Arc};

use crate::mongo::Context;
use crate::result::Result;
use once_cell::sync::OnceCell;
use pretty_assertions::assert_eq;
use tokio::sync::Mutex;

pub async fn with_mongo<Fut>(f: impl FnOnce(Arc<Context>) -> Fut) -> Result<()>
where
    Fut: std::future::Future<Output = Result<()>>,
{
    static COMMON_CONTEXT: OnceCell<Arc<Context>> = OnceCell::new();
    static MUTEX: OnceCell<Mutex<bool>> = OnceCell::new();
    let _ = MUTEX.get_or_init(|| Mutex::new(true)).lock().await;
    let ctx = match COMMON_CONTEXT.get() {
        Some(m) => m,
        None => {
            let uri = var("MONGO_TEST_URI")?;
            let database = var("MONGO_TEST_DATABASE")?;
            let m = Context::build_in_test(&uri, &database).await?;
            COMMON_CONTEXT.get_or_init(|| Arc::new(m))
        }
    };
    f(Arc::clone(ctx)).await?;
    let _ = ctx.database().drop(None).await?;
    Ok(())
}

#[tokio::test]
async fn test_with_mongo() -> Result<()> {
    with_mongo(|ctx| async move {
        assert_eq!(ctx.database().name(), var("MONGO_TEST_DATABASE")?);
        Ok(())
    })
    .await
}
