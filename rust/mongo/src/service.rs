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

pub struct WithIdCreateBehaviorDef<Repo>
where
    Repo: RepositoryWithId,
{
    p: PhantomData<fn() -> Repo>,
}

#[async_trait(?Send)]
impl<Repo> BehaveDef for WithIdCreateBehaviorDef<Repo>
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

pub struct WithIdUpdateBehaviorDef<Repo>
where
    Repo: RepositoryWithId,
{
    p: PhantomData<fn() -> Repo>,
}

#[async_trait(?Send)]
impl<Repo> BehaveDef for WithIdUpdateBehaviorDef<Repo>
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

pub struct WithIdDeleteBehaviorDef<Repo>
where
    Repo: RepositoryWithId,
{
    p: PhantomData<fn() -> Repo>,
}

#[async_trait(?Send)]
impl<Repo> BehaveDef for WithIdDeleteBehaviorDef<Repo>
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

pub struct WithIdFindOneBehaviorDef<Repo>
where
    Repo: RepositoryWithId,
{
    p: PhantomData<fn() -> Repo>,
}

#[async_trait(?Send)]
impl<Repo> BehaveDef for WithIdFindOneBehaviorDef<Repo>
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

pub struct WithIdFindManyBehaviorDef<Repo>
where
    Repo: RepositoryWithId,
{
    p: PhantomData<fn() -> Repo>,
}

#[async_trait(?Send)]
impl<Repo> BehaveDef for WithIdFindManyBehaviorDef<Repo>
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

pub type WithIdCreateBehavior<Repo> = Behave<WithIdCreateBehaviorDef<Repo>>;
pub type WithIdUpdateBehavior<Repo> = Behave<WithIdUpdateBehaviorDef<Repo>>;
pub type WithIdDeleteBehavior<Repo> = Behave<WithIdDeleteBehaviorDef<Repo>>;
pub type WithIdFindOneBehavior<Repo> = Behave<WithIdFindOneBehaviorDef<Repo>>;
pub type WithIdFindManyBehavior<Repo> = Behave<WithIdFindManyBehaviorDef<Repo>>;

pub struct WithIdBehaviors<Repo, In, Out, OnFindOneIn, OnFindManyIn>
where
    Repo: RepositoryWithId,
{
    #[allow(clippy::type_complexity)]
    p: PhantomData<fn() -> (Repo, In, Out, OnFindOneIn, OnFindManyIn)>,
}

impl<Repo, In, Out, OnFindOneIn, OnFindManyIn> CRUDBehaviors
    for WithIdBehaviors<Repo, In, Out, OnFindOneIn, OnFindManyIn>
where
    Repo: RepositoryWithId,
    WithId<Repo::Model>: Convertible<Out>,
{
    type Ctx = Repo::Ctx;
    type CreateIn = In;
    type UpdateIn = WithId<In>;
    type DeleteIn = Id;
    type FindOneIn = OnFindOneIn;
    type FindManyIn = OnFindManyIn;
    type FindOneOut = Option<Out>;
    type FindManyOut = ConvertModelWithIdCursor<Out, Repo::Model>;
    type Create = WithIdCreateBehavior<Repo>;
    type Update = WithIdUpdateBehavior<Repo>;
    type Delete = WithIdDeleteBehavior<Repo>;
    type FindOne = WithIdFindOneBehavior<Repo>;
    type FindMany = WithIdFindManyBehavior<Repo>;
}

pub type WithIdCRUDService<
    Repo,
    In,
    Out,
    OnFindOneIn,
    OnFindManyIn,
    BeforeFilter = EmptyHook<<Repo as RepositoryWithId>::Ctx>,
> = CRUDSevice<
    SimpleCRUDServiceDef<WithIdBehaviors<Repo, In, Out, OnFindOneIn, OnFindManyIn>, BeforeFilter>,
>;

impl Convertible<DeleteId> for WithHookResult<(), Id> {
    #[inline]
    fn convert(self) -> Result<DeleteId> {
        Ok(DeleteId(self.1))
    }
}
