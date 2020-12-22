use core::marker::Unpin;
use std::marker::PhantomData;
use std::pin::Pin;

use async_trait::async_trait;
use futures::{
    stream,
    task::{Context, Poll},
    Stream, StreamExt,
};
use mongodb::{options::FindOptions, Collection, Cursor};
use mongodm::{
    bson::{from_bson, oid::ObjectId, Bson, Document},
    doc,
    prelude::MongoError,
    Model, Repository,
};
use validator::Validate;

use crate::{
    context::MongodmContext,
    fcomps::{
        convert::Convertible,
        service::{FromHookResult, HookResult},
    },
    utils::{
        result::{Result, StdResult},
        simple_error,
    },
};

pub type Id = ObjectId;

#[derive(Debug)]
pub struct WithId<M>(pub Id, pub M);

impl<M> HookResult for WithId<M> {}

impl<T, U> Convertible<WithId<T>> for WithId<U>
where
    U: Convertible<T>,
{
    #[inline]
    fn convert(self) -> Result<WithId<T>> {
        Ok(WithId(self.0, self.1.convert()?))
    }
}

impl<T, U, H> FromHookResult<H, WithId<U>> for WithId<T>
where
    T: FromHookResult<H, U>,
{
    #[inline]
    fn from_hook_result(h: H, input: WithId<U>) -> Result<Self> {
        Ok(Self(input.0, T::from_hook_result(h, input.1)?))
    }
}

#[derive(Debug)]
pub struct ConvertibleStream<T, St>
where
    St: Stream + Unpin,
    St::Item: Convertible<T>,
{
    #[allow(clippy::type_complexity)]
    inner: stream::Map<St, fn(St::Item) -> Result<T>>,
}

impl<T, St> ConvertibleStream<T, St>
where
    St: Stream + Unpin,
    St::Item: Convertible<T>,
{
    fn from_stream_item(item: St::Item) -> Result<T> {
        item.convert()
    }
}

impl<T, St> From<St> for ConvertibleStream<T, St>
where
    St: Stream + Unpin,
    St::Item: Convertible<T>,
{
    #[inline]
    fn from(stream: St) -> Self {
        let f: fn(St::Item) -> Result<T> = Self::from_stream_item;
        let inner = stream.map(f);
        Self { inner }
    }
}

impl<T, St> Stream for ConvertibleStream<T, St>
where
    St: Stream + Unpin,
    St::Item: Convertible<T>,
{
    type Item = Result<T>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().inner).poll_next(cx)
    }
}

impl<M> Convertible<WithId<M>> for StdResult<Document, MongoError>
where
    M: Model,
{
    fn convert(self) -> Result<WithId<M>> {
        match self {
            Ok(doc) => {
                let id = get_id_from_doc(&doc);
                let doc = from_bson(Bson::Document(doc));
                match (id, doc) {
                    (Ok(id), Ok(doc)) => Ok(WithId(id, doc)),
                    _ => Err(simple_error!("find conv error")),
                }
            }
            Err(err) => Err(err.into()),
        }
    }
}

pub type ModelWithIdCursor<M> = ConvertibleStream<WithId<M>, Cursor>;

#[derive(Debug)]
pub struct ConvertibleTryStream<T, F, St>
where
    F: Convertible<T>,
    St: Stream<Item = Result<F>> + Unpin,
{
    #[allow(clippy::type_complexity)]
    inner: stream::Map<St, fn(St::Item) -> Result<T>>,
}

impl<T, F, St> ConvertibleTryStream<T, F, St>
where
    F: Convertible<T>,
    St: Stream<Item = Result<F>> + Unpin,
{
    fn from_stream_item(item: Result<F>) -> Result<T> {
        match item {
            Ok(i) => Ok(i.convert()?),
            Err(err) => Err(err),
        }
    }
}

impl<T, F, St> From<St> for ConvertibleTryStream<T, F, St>
where
    F: Convertible<T>,
    St: Stream<Item = Result<F>> + Unpin,
{
    #[inline]
    fn from(stream: St) -> Self {
        let f: fn(St::Item) -> Result<T> = Self::from_stream_item;
        let inner = stream.map(f);
        Self { inner }
    }
}

impl<T, F, St> Stream for ConvertibleTryStream<T, F, St>
where
    F: Convertible<T>,
    St: Stream<Item = Result<F>> + Unpin,
{
    type Item = Result<T>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().inner).poll_next(cx)
    }
}

impl<T, F, U, St> Convertible<ConvertibleTryStream<T, F, ConvertibleStream<F, St>>>
    for ConvertibleStream<F, St>
where
    F: Convertible<T>,
    St: Stream<Item = U> + Unpin,
    U: Convertible<F>,
{
    fn convert(self) -> Result<ConvertibleTryStream<T, F, ConvertibleStream<F, St>>> {
        Ok(ConvertibleTryStream::<T, F, ConvertibleStream<F, St>>::from(self))
    }
}

pub type ConvertModelWithIdCursor<T, M> = ConvertibleTryStream<T, WithId<M>, ModelWithIdCursor<M>>;

pub enum CreateOrUpdate {
    Create,
    Update(Id),
}

#[async_trait(?Send)]
pub trait Validator {
    type Model;
    type Ctx;
    async fn validate(cu: CreateOrUpdate, model: &'_ Self::Model, ctx: &'_ Self::Ctx)
        -> Result<()>;
}

pub struct DefaultValidate<Model, Ctx> {
    p: PhantomData<fn() -> (Model, Ctx)>,
}

#[async_trait(?Send)]
impl<Model, Ctx> Validator for DefaultValidate<Model, Ctx> {
    type Model = Model;
    type Ctx = Ctx;

    #[inline]
    async fn validate(
        _cu: CreateOrUpdate,
        _model: &'_ Self::Model,
        _ctx: &'_ Self::Ctx,
    ) -> Result<()> {
        Ok(())
    }
}

pub struct FromValidate<Model, Ctx, V = DefaultValidate<Model, Ctx>>
where
    Model: Validate,
    V: Validator<Model = Model, Ctx = Ctx>,
    Ctx: MongodmContext,
{
    #[allow(clippy::type_complexity)]
    p: PhantomData<fn() -> (Model, Ctx, V)>,
}

#[async_trait(?Send)]
impl<Model, Ctx, V> Validator for FromValidate<Model, Ctx, V>
where
    Model: Validate,
    V: Validator<Model = Model, Ctx = Ctx>,
    Ctx: MongodmContext,
{
    type Model = Model;
    type Ctx = Ctx;

    #[inline]
    async fn validate(cu: CreateOrUpdate, model: &Model, ctx: &Ctx) -> Result<()>
    where
        V: 'async_trait,
    {
        model.validate()?;
        V::validate(cu, model, ctx).await
    }
}

#[async_trait(?Send)]
pub trait RepositoryWithId {
    type Model: mongodm::Model;
    type Ctx;

    async fn new(ctx: &Self::Ctx) -> Self;

    async fn create(&self, model: &Self::Model) -> Result<Id>;

    async fn update(&self, model: &WithId<Self::Model>) -> Result<()>;

    async fn delete(&self, id: &Id) -> Result<()>;

    async fn find_one(&self, query: Document) -> Result<Option<WithId<Self::Model>>>;

    async fn find_one_by_id(&self, id: &Id) -> Result<Option<WithId<Self::Model>>>;

    async fn find_many(
        &self,
        query: Document,
        option: Option<FindOptions>,
    ) -> Result<ModelWithIdCursor<Self::Model>>
    where
        Self::Model: Model;
}

pub struct RepositoryWithIdBase<M, Ctx, V = DefaultValidate<M, Ctx>>
where
    Ctx: MongodmContext + Clone,
    M: Model,
    V: Validator<Model = M, Ctx = Ctx>,
{
    pub(crate) ctx: Ctx,
    pub(crate) repo: Repository<M>,
    pub(crate) coll: Collection,
    p: PhantomData<V>,
}

#[async_trait(?Send)]
impl<M, Ctx, V> RepositoryWithId for RepositoryWithIdBase<M, Ctx, V>
where
    Ctx: MongodmContext,
    M: Model,
    V: Validator<Model = M, Ctx = Ctx>,
{
    type Model = M;
    type Ctx = Ctx;

    async fn new(ctx: &Ctx) -> Self {
        let repo = ctx.repo::<M>();
        Self {
            ctx: ctx.clone(),
            repo: repo.clone(),
            coll: repo.get_underlying(),
            p: PhantomData,
        }
    }

    async fn create(&self, model: &M) -> Result<Id> {
        V::validate(CreateOrUpdate::Create, model, &self.ctx).await?;
        oid(self.repo.insert_one(model, None).await?.inserted_id)
    }

    async fn update(&self, model: &WithId<M>) -> Result<()> {
        V::validate(CreateOrUpdate::Update(model.0.clone()), &model.1, &self.ctx).await?;
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

    async fn delete(&self, id: &Id) -> Result<()> {
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

    async fn find_one(&self, query: Document) -> Result<Option<WithId<M>>> {
        let doc_opt = self.coll.find_one(query, None).await?;
        if let Some(doc) = doc_opt {
            let id = get_id_from_doc(&doc)?;
            let item = h_doc_to_model(doc)?;
            Ok(Some(WithId(id, item)))
        } else {
            Ok(None)
        }
    }

    async fn find_one_by_id(&self, id: &Id) -> Result<Option<WithId<M>>> {
        self.find_one(doc! {"_id": Bson::ObjectId(id.clone())})
            .await
    }

    async fn find_many(
        &self,
        query: Document,
        option: Option<FindOptions>,
    ) -> Result<ModelWithIdCursor<M>> {
        Ok(ModelWithIdCursor::from(
            self.coll.find(query, option).await?,
        ))
    }
}

fn oid(bson: Bson) -> Result<Id> {
    match bson {
        Bson::ObjectId(oid) => Ok(oid),
        _ => Err(simple_error!("value is not ObjectId")),
    }
}

fn get_id_from_doc(doc: &Document) -> Result<Id> {
    if let Some(id) = doc.get("_id") {
        oid(id.clone())
    } else {
        Err(simple_error!("document don'n have _id"))
    }
}

fn h_doc_to_model<M>(doc: Document) -> Result<M>
where
    M: Model,
{
    Ok(from_bson(Bson::Document(doc))?)
}

pub type ValidatedRepositoryWithId<M, Ctx, V = DefaultValidate<M, Ctx>> =
    RepositoryWithIdBase<M, Ctx, FromValidate<M, Ctx, V>>;

pub async fn _valiidate_uniqueness<Repo>(
    doc_on_create: Document,
    doc_on_update: impl Fn(Id) -> Document,
    repo: &Repo,
    cu: CreateOrUpdate,
) -> Result<()>
where
    Repo: RepositoryWithId,
{
    match cu {
        CreateOrUpdate::Create => {
            if repo.find_one(doc_on_create).await?.is_some() {
                Err(simple_error!("not unique on create"))
            } else {
                Ok(())
            }
        }
        CreateOrUpdate::Update(id) => {
            if repo.find_one(doc_on_update(id)).await?.is_some() {
                Err(simple_error!("not unique on update"))
            } else {
                Ok(())
            }
        }
    }
}

#[macro_export]
macro_rules! validate_uniqueness {
    (<$repo:ty> [$field:ident] $cu:tt, $model:tt, $ctx:tt) => {
        let repo = <$repo>::new($ctx).await;
        $crate::withid::_valiidate_uniqueness(
            mongodm::doc! {
                stringify!($field): &$model.$field
            },
            |id| {
                mongodm::doc! {
                    "_id": {mongodm::operator::Not: {mongodm::operator::Equal: id}},
                    stringify!($field): &$model.$field
                }
            },
            &repo,
            $cu,
        )
        .await?
    };
}
