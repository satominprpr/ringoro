// TODO: serde が実装されていればOKにしたい
//       Mongodm には非依存になるしかないかも
use std::marker::PhantomData;

use crate::{
    behavior::{lift::Lift, Behavior, NoBehave},
    convert::{Convert, Convertible, Identity},
    result::Result,
    service::servicebase::*,
    Callable, SeqB, Through,
};

pub trait CRUDSeviceDef {
    type Ctx;
    type CreateDef: ServiceBaseDef<Ctx = Self::Ctx>;
    type UpdateDef: ServiceBaseDef<Ctx = Self::Ctx>;
    type DeleteDef: ServiceBaseDef<Ctx = Self::Ctx>;
    type FindOneDef: ServiceBaseDef<Ctx = Self::Ctx>;
    type FindManyDef: ServiceBaseDef<Ctx = Self::Ctx>;
}

pub struct CRUDSevice<Def>
where
    Def: CRUDSeviceDef,
{
    p: PhantomData<Def>,
}

impl<Def> CRUDSevice<Def>
where
    Def: CRUDSeviceDef,
{
    #[inline]
    pub async fn create(
        i: <<Def as CRUDSeviceDef>::CreateDef as ServiceBaseDef>::In,
        ctx: &<<Def as CRUDSeviceDef>::CreateDef as ServiceBaseDef>::Ctx,
    ) -> Result<<<Def as CRUDSeviceDef>::CreateDef as ServiceBaseDef>::Out> {
        ServiceBase::<Def::CreateDef>::apply(i, ctx).await.result()
    }

    #[inline]
    pub async fn update(
        i: <<Def as CRUDSeviceDef>::UpdateDef as ServiceBaseDef>::In,
        ctx: &<<Def as CRUDSeviceDef>::UpdateDef as ServiceBaseDef>::Ctx,
    ) -> Result<<<Def as CRUDSeviceDef>::UpdateDef as ServiceBaseDef>::Out> {
        ServiceBase::<Def::UpdateDef>::apply(i, ctx).await.result()
    }

    #[inline]
    pub async fn delete(
        i: <<Def as CRUDSeviceDef>::DeleteDef as ServiceBaseDef>::In,
        ctx: &<<Def as CRUDSeviceDef>::DeleteDef as ServiceBaseDef>::Ctx,
    ) -> Result<<<Def as CRUDSeviceDef>::DeleteDef as ServiceBaseDef>::Out> {
        ServiceBase::<Def::DeleteDef>::apply(i, ctx).await.result()
    }

    #[inline]
    pub async fn find_one(
        i: <<Def as CRUDSeviceDef>::FindOneDef as ServiceBaseDef>::In,
        ctx: &<<Def as CRUDSeviceDef>::FindOneDef as ServiceBaseDef>::Ctx,
    ) -> Result<<<Def as CRUDSeviceDef>::FindOneDef as ServiceBaseDef>::Out> {
        ServiceBase::<Def::FindOneDef>::apply(i, ctx).await.result()
    }

    #[inline]
    pub async fn find_many(
        i: <<Def as CRUDSeviceDef>::FindManyDef as ServiceBaseDef>::In,
        ctx: &<<Def as CRUDSeviceDef>::FindManyDef as ServiceBaseDef>::Ctx,
    ) -> Result<<<Def as CRUDSeviceDef>::FindManyDef as ServiceBaseDef>::Out> {
        ServiceBase::<Def::FindManyDef>::apply(i, ctx)
            .await
            .result()
    }
}

pub trait CRUDBehaviors {
    type Ctx;
    type CreateIn;
    type UpdateIn;
    type DeleteIn;
    type FindOneIn;
    type FindManyIn;
    type FindOneOut;
    type FindManyOut;
    type Create: Behavior<Ctx = Self::Ctx>;
    type Update: Behavior<Ctx = Self::Ctx>;
    type Delete: Behavior<Ctx = Self::Ctx>;
    type FindOne: Behavior<Ctx = Self::Ctx>;
    type FindMany: Behavior<Ctx = Self::Ctx>;
}

pub trait CRUDHook {
    type Ctx;
    type HookOut;
    type Hook: Behavior<In = (), Out = Self::HookOut, Ctx = Self::Ctx>;
    type OnCreate: Callable<In = Self::HookOut, Out = Self::HookOut>;
    type OnUpdate: Callable<In = Self::HookOut, Out = Self::HookOut>;
    type OnDelete: Callable<In = Self::HookOut, Out = Self::HookOut>;
    type OnFindOne: Callable<In = Self::HookOut, Out = Self::HookOut>;
    type OnFindMany: Callable<In = Self::HookOut, Out = Self::HookOut>;
}

pub struct EmptyHook<Ctx> {
    p: PhantomData<fn() -> Ctx>,
}

impl<Ctx> CRUDHook for EmptyHook<Ctx> {
    type Ctx = Ctx;
    type HookOut = ();
    type Hook = NoBehave<(), Ctx>;
    type OnCreate = Through<()>;
    type OnUpdate = Through<()>;
    type OnDelete = Through<()>;
    type OnFindOne = Through<()>;
    type OnFindMany = Through<()>;
}

pub struct SimpleCRUDServiceDef<
    Behaviors,
    BeforeFilter = EmptyHook<<Behaviors as CRUDBehaviors>::Ctx>,
> where
    Behaviors: CRUDBehaviors,
    BeforeFilter: CRUDHook<Ctx = <Behaviors as CRUDBehaviors>::Ctx>,
    WithHookResult<<BeforeFilter as CRUDHook>::HookOut, <Behaviors as CRUDBehaviors>::CreateIn>:
        Convertible<<<Behaviors as CRUDBehaviors>::Create as Behavior>::In>,
    WithHookResult<<BeforeFilter as CRUDHook>::HookOut, <Behaviors as CRUDBehaviors>::UpdateIn>:
        Convertible<<<Behaviors as CRUDBehaviors>::Update as Behavior>::In>,
{
    #[allow(clippy::type_complexity)]
    p: PhantomData<fn() -> (Behaviors, BeforeFilter)>,
}

impl<Behaviors, BeforeFilter> CRUDSeviceDef for SimpleCRUDServiceDef<Behaviors, BeforeFilter>
where
    Behaviors: CRUDBehaviors,
    BeforeFilter: CRUDHook<Ctx = <Behaviors as CRUDBehaviors>::Ctx>,
    WithHookResult<<BeforeFilter as CRUDHook>::HookOut, <Behaviors as CRUDBehaviors>::CreateIn>:
        Convertible<<<Behaviors as CRUDBehaviors>::Create as Behavior>::In>,
    WithHookResult<<BeforeFilter as CRUDHook>::HookOut, <Behaviors as CRUDBehaviors>::UpdateIn>:
        Convertible<<<Behaviors as CRUDBehaviors>::Update as Behavior>::In>,
    WithHookResult<<BeforeFilter as CRUDHook>::HookOut, <Behaviors as CRUDBehaviors>::DeleteIn>:
        Convertible<<<Behaviors as CRUDBehaviors>::Delete as Behavior>::In>,
    WithHookResult<<BeforeFilter as CRUDHook>::HookOut, <Behaviors as CRUDBehaviors>::FindOneIn>:
        Convertible<<<Behaviors as CRUDBehaviors>::FindOne as Behavior>::In>,
    <<Behaviors as CRUDBehaviors>::FindOne as Behavior>::Out:
        Convertible<<Behaviors as CRUDBehaviors>::FindOneOut>,
    WithHookResult<<BeforeFilter as CRUDHook>::HookOut, <Behaviors as CRUDBehaviors>::FindManyIn>:
        Convertible<<<Behaviors as CRUDBehaviors>::FindMany as Behavior>::In>,
    <<Behaviors as CRUDBehaviors>::FindMany as Behavior>::Out:
        Convertible<<Behaviors as CRUDBehaviors>::FindManyOut>,
{
    type Ctx = Behaviors::Ctx;

    #[allow(clippy::type_complexity)]
    type CreateDef = ServiceBaseBuild<
        <Behaviors as CRUDBehaviors>::CreateIn,
        SeqB!(
            <BeforeFilter as CRUDHook>::Hook,
            Lift<<BeforeFilter as CRUDHook>::OnCreate, Self::Ctx>
        ),
        Convert<
            WithHookResult<
                <BeforeFilter as CRUDHook>::HookOut,
                <Behaviors as CRUDBehaviors>::CreateIn,
            >,
            <<Behaviors as CRUDBehaviors>::Create as Behavior>::In,
        >,
        Behaviors::Create,
        Identity<<<Behaviors as CRUDBehaviors>::Create as Behavior>::Out>,
        <<Behaviors as CRUDBehaviors>::Create as Behavior>::Out,
    >;

    #[allow(clippy::type_complexity)]
    type UpdateDef = ServiceBaseBuild<
        <Behaviors as CRUDBehaviors>::UpdateIn,
        SeqB!(
            <BeforeFilter as CRUDHook>::Hook,
            Lift<<BeforeFilter as CRUDHook>::OnUpdate, Self::Ctx>
        ),
        Convert<
            WithHookResult<
                <BeforeFilter as CRUDHook>::HookOut,
                <Behaviors as CRUDBehaviors>::UpdateIn,
            >,
            <<Behaviors as CRUDBehaviors>::Update as Behavior>::In,
        >,
        Behaviors::Update,
        Identity<<<Behaviors as CRUDBehaviors>::Update as Behavior>::Out>,
        <<Behaviors as CRUDBehaviors>::Update as Behavior>::Out,
    >;

    #[allow(clippy::type_complexity)]
    type DeleteDef = ServiceBaseBuild<
        <Behaviors as CRUDBehaviors>::DeleteIn,
        SeqB!(
            <BeforeFilter as CRUDHook>::Hook,
            Lift<<BeforeFilter as CRUDHook>::OnDelete, Self::Ctx>
        ),
        Convert<
            WithHookResult<
                <BeforeFilter as CRUDHook>::HookOut,
                <Behaviors as CRUDBehaviors>::DeleteIn,
            >,
            <<Behaviors as CRUDBehaviors>::Delete as Behavior>::In,
        >,
        Behaviors::Delete,
        Identity<<<Behaviors as CRUDBehaviors>::Delete as Behavior>::Out>,
        <<Behaviors as CRUDBehaviors>::Delete as Behavior>::Out,
    >;

    #[allow(clippy::type_complexity)]
    type FindOneDef = ServiceBaseBuild<
        <Behaviors as CRUDBehaviors>::FindOneIn,
        SeqB!(
            <BeforeFilter as CRUDHook>::Hook,
            Lift<<BeforeFilter as CRUDHook>::OnFindOne, Self::Ctx>
        ),
        Convert<
            WithHookResult<
                <BeforeFilter as CRUDHook>::HookOut,
                <Behaviors as CRUDBehaviors>::FindOneIn,
            >,
            <<Behaviors as CRUDBehaviors>::FindOne as Behavior>::In,
        >,
        Behaviors::FindOne,
        Convert<
            <<Behaviors as CRUDBehaviors>::FindOne as Behavior>::Out,
            <Behaviors as CRUDBehaviors>::FindOneOut,
        >,
        <Behaviors as CRUDBehaviors>::FindOneOut,
    >;

    #[allow(clippy::type_complexity)]
    type FindManyDef = ServiceBaseBuild<
        <Behaviors as CRUDBehaviors>::FindManyIn,
        SeqB!(
            <BeforeFilter as CRUDHook>::Hook,
            Lift<<BeforeFilter as CRUDHook>::OnFindMany, Self::Ctx>
        ),
        Convert<
            WithHookResult<
                <BeforeFilter as CRUDHook>::HookOut,
                <Behaviors as CRUDBehaviors>::FindManyIn,
            >,
            <<Behaviors as CRUDBehaviors>::FindMany as Behavior>::In,
        >,
        Behaviors::FindMany,
        Convert<
            <<Behaviors as CRUDBehaviors>::FindMany as Behavior>::Out,
            <Behaviors as CRUDBehaviors>::FindManyOut,
        >,
        <Behaviors as CRUDBehaviors>::FindManyOut,
    >;
}
