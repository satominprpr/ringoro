use core::pin::Pin;
use futures::{
    task::{Context, Poll},
    Stream,
};
use mongodb::{options::FindOptions, Collection};
use mongodm::{
    bson::{from_bson, oid::ObjectId, Bson, Document},
    doc, Model, Repository,
};

use crate::{
    context::MongodmContext,
    utils::{result::Result, simple_error},
};

type Id = ObjectId;

pub struct RepositoryWithId<M>
where
    M: Model,
{
    repo: Repository<M>,
    coll: Collection,
}

impl<M> RepositoryWithId<M>
where
    M: Model,
{
    pub fn new(ctx: &impl MongodmContext) -> Self {
        let repo = ctx.repo::<M>();
        Self {
            repo: repo.clone(),
            coll: repo.get_underlying(),
        }
    }

    pub async fn create(&self, model: &M) -> Result<Id> {
        Self::oid(self.repo.insert_one(model, None).await?.inserted_id)
    }

    pub async fn update(&self, model: &(Id, M)) -> Result<()> {
        let result = self
            .repo
            .replace_one(
                doc! {"_id": Bson::ObjectId(model.0.clone()) },
                &model.1,
                None,
            )
            .await?;
        match (result.modified_count, result.matched_count) {
            (1, 1) => Ok(()),
            (0, 0) => Err(simple_error!("not found!")), // TODO: 404
            _ => Err(simple_error!("cannot update!")),
        }
    }

    pub async fn delete(&self, id: &Id) -> Result<()> {
        let result = self
            .repo
            .delete_one(doc! {"_id": Bson::ObjectId(id.clone()) }, None)
            .await?;
        if result.deleted_count == 1 {
            Ok(())
        } else {
            Err(simple_error!("cannot update!"))
        }
    }

    pub async fn find_one(&self, query: Document) -> Result<Option<(Id, M)>> {
        let doc_opt = self.coll.find_one(query, None).await?;
        if let Some(doc) = doc_opt {
            let id = Self::get_id_from_doc(&doc)?;
            let item = Self::h_doc_to_model(doc)?;
            Ok(Some((id, item)))
        } else {
            Ok(None)
        }
    }

    pub async fn find_one_by_id(&self, id: &Id) -> Result<Option<(Id, M)>> {
        self.find_one(doc! {"_id": Bson::ObjectId(id.clone())})
            .await
    }

    pub async fn find(
        &self,
        query: Document,
        option: impl Into<Option<FindOptions>>,
    ) -> Result<ModelWithIdCursor<M>> {
        Ok(ModelWithIdCursor::from(
            self.coll.find(query, option).await?,
        ))
    }

    pub(crate) fn oid(bson: Bson) -> Result<Id> {
        match bson {
            Bson::ObjectId(oid) => Ok(oid),
            _ => Err(simple_error!("value is not ObjectId")),
        }
    }

    pub(crate) fn get_id_from_doc(doc: &Document) -> Result<Id> {
        if let Some(id) = doc.get("_id") {
            Self::oid(id.clone())
        } else {
            Err(simple_error!("document don'n have _id"))
        }
    }

    pub(crate) fn h_doc_to_model(doc: Document) -> Result<M> {
        Ok(from_bson(Bson::Document(doc))?)
    }
}

#[derive(Debug)]
pub struct ModelWithIdCursor<M: Model> {
    inner: mongodb::Cursor,
    _pd: std::marker::PhantomData<M>,
}

impl<M: Model> From<mongodb::Cursor> for ModelWithIdCursor<M> {
    fn from(inner: mongodb::Cursor) -> Self {
        Self {
            inner,
            _pd: std::marker::PhantomData,
        }
    }
}

impl<M: Model + Unpin> Stream for ModelWithIdCursor<M> {
    type Item = Result<(Id, M)>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Result<(Id, M)>>> {
        match Pin::new(&mut self.get_mut().inner).poll_next(cx) {
            Poll::Ready(Some(Ok(doc))) => {
                let id = RepositoryWithId::<M>::get_id_from_doc(&doc);
                let doc = from_bson(Bson::Document(doc));
                let res = match (id, doc) {
                    (Ok(id), Ok(doc)) => Ok((id, doc)),
                    _ => Err(simple_error!("find conv error")),
                };
                Poll::Ready(Some(res))
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e.into()))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

#[cfg(test)]
mod test {
    use futures::TryStreamExt;
    use mongodm::{
        operator::*,
        {f, CollectionConfig, Indexes},
    };
    use once_cell::sync::Lazy;
    use pretty_assertions::assert_eq;
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::{context::MongodmContext, test_util::with_mongo};

    #[derive(Serialize, Deserialize)]
    struct TricoUnit {
        name: String,
        cu: String,
        co: String,
        pa: String,
    }

    struct TricoUnitCfg {}

    impl CollectionConfig for TricoUnitCfg {
        fn collection_name() -> &'static str {
            "TricoUnit"
        }

        fn indexes() -> Indexes {
            Indexes::new()
        }
    }

    impl Model for TricoUnit {
        type CollConf = TricoUnitCfg;
    }

    type Repo = RepositoryWithId<TricoUnit>;

    static UNIBO: Lazy<TricoUnit> = Lazy::new(|| TricoUnit {
        name: "unibo".to_owned(),
        cu: "akari".to_owned(),
        co: "akira".to_owned(),
        pa: "riamu".to_owned(),
    });
    static NW: Lazy<TricoUnit> = Lazy::new(|| TricoUnit {
        name: "nw".to_owned(),
        cu: "sakura".to_owned(),
        co: "izumi".to_owned(),
        pa: "ako".to_owned(),
    });
    static NG: Lazy<TricoUnit> = Lazy::new(|| TricoUnit {
        name: "ng".to_owned(),
        cu: "uzuki".to_owned(),
        co: "rin".to_owned(),
        pa: "mio".to_owned(),
    });

    #[tokio::test]
    async fn test_create() {
        with_mongo(|ctx| async move {
            let grepo = ctx.repo::<TricoUnit>();
            let repo = Repo::new(ctx.as_ref());
            let oid = repo.create(&UNIBO).await.unwrap();
            let result = grepo
                .find_one(doc! { "_id": Bson::ObjectId(oid)}, None)
                .await
                .unwrap()
                .unwrap();
            assert_eq!("akari", result.cu);
            Ok(())
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_update() {
        with_mongo(|ctx| async move {
            let grepo = ctx.repo::<TricoUnit>();
            let repo = Repo::new(ctx.as_ref());
            let oid = grepo.insert_one(&UNIBO, None).await.unwrap().inserted_id;
            let oid = Bson::as_object_id(&oid).unwrap();
            let _ = grepo.insert_one(&NW, None).await.unwrap();
            let m = TricoUnit {
                name: "unibo".to_owned(),
                cu: "akari".to_owned(),
                co: "akira".to_owned(),
                pa: "--dekin--".to_owned(),
            };
            repo.update(&(oid.clone(), m)).await.unwrap();
            let c = grepo.count_documents(None, None).await.unwrap();
            assert_eq!(2, c);
            let result = grepo
                .find_one(doc! { "_id": Bson::ObjectId(oid.clone())}, None)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(result.pa, "--dekin--");
            Ok(())
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_delete() {
        with_mongo(|ctx| async move {
            let grepo = ctx.repo::<TricoUnit>();
            let repo = Repo::new(ctx.as_ref());
            let oid = grepo.insert_one(&UNIBO, None).await.unwrap().inserted_id;
            let oid = Bson::as_object_id(&oid).unwrap();
            let noid = grepo.insert_one(&NW, None).await.unwrap().inserted_id;
            let noid = Bson::as_object_id(&noid).unwrap();
            repo.delete(&oid).await.unwrap();
            let c = grepo.count_documents(None, None).await.unwrap();
            assert_eq!(1, c);

            let result = grepo
                .find_one(doc! { "_id": Bson::ObjectId(noid.clone())}, None)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(result.cu, "sakura");
            Ok(())
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_find_one() {
        with_mongo(|ctx| async move {
            let grepo = ctx.repo::<TricoUnit>();
            let repo = Repo::new(ctx.as_ref());
            let id = grepo.insert_one(&UNIBO, None).await.unwrap().inserted_id;
            let id = Bson::as_object_id(&id).unwrap();
            let _ = grepo.insert_one(&NW, None).await.unwrap().inserted_id;
            let (rid, result) = repo
                .find_one(doc! { f!(cu in TricoUnit): "akari"})
                .await?
                .unwrap();
            assert_eq!(id, &rid);
            assert_eq!("akira", result.co);
            Ok(())
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_find_one_by_id() {
        with_mongo(|ctx| async move {
            let grepo = ctx.repo::<TricoUnit>();
            let repo = Repo::new(ctx.as_ref());
            let id = grepo.insert_one(&UNIBO, None).await.unwrap().inserted_id;
            let id = Bson::as_object_id(&id).unwrap();
            let _ = grepo.insert_one(&NW, None).await.unwrap().inserted_id;
            let (rid, result) = repo.find_one_by_id(&id).await?.unwrap();
            assert_eq!(id, &rid);
            assert_eq!("akira", result.co);
            Ok(())
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_find() {
        with_mongo(|ctx| async move {
            let grepo = ctx.repo::<TricoUnit>();
            let repo = Repo::new(ctx.as_ref());
            let unibo_id = grepo.insert_one(&UNIBO, None).await.unwrap().inserted_id;
            let unibo_id = Bson::as_object_id(&unibo_id).unwrap();
            let nw_id = grepo.insert_one(&NW, None).await.unwrap().inserted_id;
            let nw_id = Bson::as_object_id(&nw_id).unwrap();
            let _ = grepo.insert_one(&NG, None).await.unwrap().inserted_id;
            let opt = FindOptions::builder()
                .sort(doc! { f!(name in TricoUnit): -1})
                .build();
            let result = repo
                .find(
                    doc! {
                        Or: [
                        { f!(cu in TricoUnit): "akari"},
                        { f!(cu in TricoUnit): "sakura"},
                        ]
                    },
                    opt,
                )
                .await?
                .try_collect::<Vec<_>>()
                .await?;
            assert_eq!(2, result.len());
            assert_eq!(unibo_id, &result[0].0);
            assert_eq!(nw_id, &result[1].0);
            Ok(())
        })
        .await
        .unwrap()
    }
}
