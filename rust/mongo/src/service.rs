use std::marker::PhantomData;

use async_trait::async_trait;
use bson::Document;
use mongodm::mongo::options::FindOptions;

use crate::{
    fcomps::{
        behavior::{Behave, BehaveDef},
        convert::Convertible,
        service::*,
    },
    utils::result::Result,
    withid::{
        ConvertModelWithIdCursor, ConvertibleStream, Id, ModelWithIdCursor, RepositoryWithId,
        WithId,
    },
};

pub struct WithIdCreateBehavorDef<Repo>
where
    Repo: RepositoryWithId,
{
    p: PhantomData<fn() -> Repo>,
}

#[async_trait(?Send)]
impl<Repo> BehaveDef for WithIdCreateBehavorDef<Repo>
where
    Repo: RepositoryWithId,
{
    type In = Repo::Model;
    type Out = Id;
    type Ctx = Repo::Ctx;

    #[inline]
    async fn def(input: Self::In, ctx: &Self::Ctx) -> Result<Self::Out> {
        <Repo as RepositoryWithId>::new(ctx)
            .await
            .create(&input)
            .await
    }
}

pub struct WithIdUpdateBehavorDef<Repo>
where
    Repo: RepositoryWithId,
{
    p: PhantomData<fn() -> Repo>,
}

#[async_trait(?Send)]
impl<Repo> BehaveDef for WithIdUpdateBehavorDef<Repo>
where
    Repo: RepositoryWithId,
{
    type In = WithId<Repo::Model>;
    type Out = ();
    type Ctx = Repo::Ctx;

    #[inline]
    async fn def(input: Self::In, ctx: &Self::Ctx) -> Result<Self::Out> {
        <Repo as RepositoryWithId>::new(ctx)
            .await
            .update(&input)
            .await
    }
}

pub struct DeleteId(pub Id);

pub struct WithIdDeleteBehavorDef<Repo>
where
    Repo: RepositoryWithId,
{
    p: PhantomData<fn() -> Repo>,
}

#[async_trait(?Send)]
impl<Repo> BehaveDef for WithIdDeleteBehavorDef<Repo>
where
    Repo: RepositoryWithId,
{
    type In = DeleteId;
    type Out = ();
    type Ctx = Repo::Ctx;

    #[inline]
    async fn def(input: Self::In, ctx: &Self::Ctx) -> Result<Self::Out> {
        <Repo as RepositoryWithId>::new(ctx)
            .await
            .delete(&input.0)
            .await
    }
}

pub struct FindOneArgument(pub Document, pub Option<FindOptions>);

pub struct WithIdFindOneBehavorDef<Repo>
where
    Repo: RepositoryWithId,
{
    p: PhantomData<fn() -> Repo>,
}

#[async_trait(?Send)]
impl<Repo> BehaveDef for WithIdFindOneBehavorDef<Repo>
where
    Repo: RepositoryWithId,
{
    type In = FindOneArgument;
    type Out = Option<WithId<Repo::Model>>;
    type Ctx = Repo::Ctx;

    #[inline]
    async fn def(input: Self::In, ctx: &Self::Ctx) -> Result<Self::Out> {
        <Repo as RepositoryWithId>::new(ctx)
            .await
            .find_one(input.0)
            .await
    }
}

pub struct FindManyArgument(pub Document, pub Option<FindOptions>);

pub struct WithIdFindManyBehavorDef<Repo>
where
    Repo: RepositoryWithId,
{
    p: PhantomData<fn() -> Repo>,
}

#[async_trait(?Send)]
impl<Repo> BehaveDef for WithIdFindManyBehavorDef<Repo>
where
    Repo: RepositoryWithId,
{
    type In = FindManyArgument;
    type Out = ModelWithIdCursor<Repo::Model>;
    type Ctx = Repo::Ctx;

    #[inline]
    async fn def(input: Self::In, ctx: &Self::Ctx) -> Result<Self::Out> {
        <Repo as RepositoryWithId>::new(ctx)
            .await
            .find_many(input.0, input.1)
            .await
    }
}

pub type ConvertCursur<T, F> = ConvertibleStream<T, ModelWithIdCursor<F>>;

pub struct WithIdBehavors<Repo, In, Out, OnFindOneIn, OnFindManyIn>
where
    Repo: RepositoryWithId,
{
    #[allow(clippy::type_complexity)]
    p: PhantomData<fn() -> (Repo, In, Out, OnFindOneIn, OnFindManyIn)>,
}

impl<Repo, In, Out, OnFindOneIn, OnFindManyIn> CRUDBehaviors
    for WithIdBehavors<Repo, In, Out, OnFindOneIn, OnFindManyIn>
where
    Repo: RepositoryWithId,
    WithId<Repo::Model>: Convertible<Out>,
{
    type Ctx = Repo::Ctx;
    type OnCreateIn = In;
    type OnUpdateIn = WithId<In>;
    type OnDeleteIn = Id;
    type OnFindOneIn = OnFindOneIn;
    type OnFindManyIn = OnFindManyIn;
    type OnFindOneOut = Option<Out>;
    type OnFindManyOut = ConvertModelWithIdCursor<Out, Repo::Model>;
    type OnCreate = Behave<WithIdCreateBehavorDef<Repo>>;
    type OnUpdate = Behave<WithIdUpdateBehavorDef<Repo>>;
    type OnDelete = Behave<WithIdDeleteBehavorDef<Repo>>;
    type OnFindOne = Behave<WithIdFindOneBehavorDef<Repo>>;
    type OnFindMany = Behave<WithIdFindManyBehavorDef<Repo>>;
}

pub type WithIdCRUDService<
    Repo,
    In,
    Out,
    OnFindOneIn,
    OnFindManyIn,
    BeforeFilter = EmptyHook<<Repo as RepositoryWithId>::Ctx>,
> = CRUDSevice<
    SimpleCRUDServiceDef<WithIdBehavors<Repo, In, Out, OnFindOneIn, OnFindManyIn>, BeforeFilter>,
>;

impl Convertible<DeleteId> for WithHookResult<(), Id> {
    #[inline]
    fn convert(self) -> Result<DeleteId> {
        Ok(DeleteId(self.1))
    }
}
