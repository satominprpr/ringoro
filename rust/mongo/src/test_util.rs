use std::{env::var, sync::Arc};

use crate::context::Context;
use crate::utils::result::Result;

pub async fn with_mongo<Fut>(f: impl FnOnce(Arc<Context>) -> Fut) -> Result<()>
where
    Fut: std::future::Future<Output = Result<()>>,
{
    let ctx = {
        let uri = var("MONGO_TEST_URI")?;
        let database = var("MONGO_TEST_DATABASE")?;
        let m = Context::build_in_test(&uri, &database).await?;
        Arc::new(m)
    };
    let _ = f(Arc::clone(&ctx)).await;
    let _ = ctx.database().drop(None).await?;
    Ok(())
}

#[tokio::test]
async fn test_with_mongo() -> Result<()> {
    use mongodb::bson::doc;
    use pretty_assertions::assert_eq;

    with_mongo(|ctx| async move {
        assert_eq!(ctx.database().name(), var("MONGO_TEST_DATABASE")?);
        let coll = ctx.database().collection("hoge");
        let docs = vec![
            doc! { "title": "1984", "author": "George Orwell" },
            doc! { "title": "Animal Farm", "author": "George Orwell" },
            doc! { "title": "The Great Gatsby", "author": "F. Scott Fitzgerald" },
        ];
        coll.insert_many(docs, None).await?;
        let count = coll.count_documents(None, None).await?;
        assert_eq!(3, count);
        Ok(())
    })
    .await?;
    let ctx = {
        let uri = var("MONGO_TEST_URI")?;
        let database = var("MONGO_TEST_DATABASE")?;
        let m = Context::build_in_test(&uri, &database).await?;
        Arc::new(m)
    };
    let count = ctx
        .database()
        .collection("hoge")
        .count_documents(None, None)
        .await?;
    assert_eq!(0, count);
    Ok(())
}
