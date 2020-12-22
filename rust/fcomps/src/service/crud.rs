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
    type OnCreateIn;
    type OnUpdateIn;
    type OnDeleteIn;
    type OnFindOneIn;
    type OnFindManyIn;
    type OnFindOneOut;
    type OnFindManyOut;
    type OnCreate: Behavior<Ctx = Self::Ctx>;
    type OnUpdate: Behavior<Ctx = Self::Ctx>;
    type OnDelete: Behavior<Ctx = Self::Ctx>;
    type OnFindOne: Behavior<Ctx = Self::Ctx>;
    type OnFindMany: Behavior<Ctx = Self::Ctx>;
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
    WithHookResult<<BeforeFilter as CRUDHook>::HookOut, <Behaviors as CRUDBehaviors>::OnCreateIn>:
        Convertible<<<Behaviors as CRUDBehaviors>::OnCreate as Behavior>::In>,
    WithHookResult<<BeforeFilter as CRUDHook>::HookOut, <Behaviors as CRUDBehaviors>::OnUpdateIn>:
        Convertible<<<Behaviors as CRUDBehaviors>::OnUpdate as Behavior>::In>,
{
    #[allow(clippy::type_complexity)]
    p: PhantomData<fn() -> (Behaviors, BeforeFilter)>,
}

impl<Behaviors, BeforeFilter> CRUDSeviceDef for SimpleCRUDServiceDef<Behaviors, BeforeFilter>
where
    Behaviors: CRUDBehaviors,
    BeforeFilter: CRUDHook<Ctx = <Behaviors as CRUDBehaviors>::Ctx>,
    WithHookResult<<BeforeFilter as CRUDHook>::HookOut, <Behaviors as CRUDBehaviors>::OnCreateIn>:
        Convertible<<<Behaviors as CRUDBehaviors>::OnCreate as Behavior>::In>,
    WithHookResult<<BeforeFilter as CRUDHook>::HookOut, <Behaviors as CRUDBehaviors>::OnUpdateIn>:
        Convertible<<<Behaviors as CRUDBehaviors>::OnUpdate as Behavior>::In>,
    WithHookResult<<BeforeFilter as CRUDHook>::HookOut, <Behaviors as CRUDBehaviors>::OnDeleteIn>:
        Convertible<<<Behaviors as CRUDBehaviors>::OnDelete as Behavior>::In>,
    WithHookResult<<BeforeFilter as CRUDHook>::HookOut, <Behaviors as CRUDBehaviors>::OnFindOneIn>:
        Convertible<<<Behaviors as CRUDBehaviors>::OnFindOne as Behavior>::In>,
    <<Behaviors as CRUDBehaviors>::OnFindOne as Behavior>::Out:
        Convertible<<Behaviors as CRUDBehaviors>::OnFindOneOut>,
    WithHookResult<<BeforeFilter as CRUDHook>::HookOut, <Behaviors as CRUDBehaviors>::OnFindManyIn>:
        Convertible<<<Behaviors as CRUDBehaviors>::OnFindMany as Behavior>::In>,
    <<Behaviors as CRUDBehaviors>::OnFindMany as Behavior>::Out:
        Convertible<<Behaviors as CRUDBehaviors>::OnFindManyOut>,
{
    type Ctx = Behaviors::Ctx;

    #[allow(clippy::type_complexity)]
    type CreateDef = ServiceBaseBuild<
        <Behaviors as CRUDBehaviors>::OnCreateIn,
        SeqB!(
            <BeforeFilter as CRUDHook>::Hook,
            Lift<<BeforeFilter as CRUDHook>::OnCreate, Self::Ctx>
        ),
        Convert<
            WithHookResult<
                <BeforeFilter as CRUDHook>::HookOut,
                <Behaviors as CRUDBehaviors>::OnCreateIn,
            >,
            <<Behaviors as CRUDBehaviors>::OnCreate as Behavior>::In,
        >,
        Behaviors::OnCreate,
        Identity<<<Behaviors as CRUDBehaviors>::OnCreate as Behavior>::Out>,
        <<Behaviors as CRUDBehaviors>::OnCreate as Behavior>::Out,
    >;

    #[allow(clippy::type_complexity)]
    type UpdateDef = ServiceBaseBuild<
        <Behaviors as CRUDBehaviors>::OnUpdateIn,
        SeqB!(
            <BeforeFilter as CRUDHook>::Hook,
            Lift<<BeforeFilter as CRUDHook>::OnUpdate, Self::Ctx>
        ),
        Convert<
            WithHookResult<
                <BeforeFilter as CRUDHook>::HookOut,
                <Behaviors as CRUDBehaviors>::OnUpdateIn,
            >,
            <<Behaviors as CRUDBehaviors>::OnUpdate as Behavior>::In,
        >,
        Behaviors::OnUpdate,
        Identity<<<Behaviors as CRUDBehaviors>::OnUpdate as Behavior>::Out>,
        <<Behaviors as CRUDBehaviors>::OnUpdate as Behavior>::Out,
    >;

    #[allow(clippy::type_complexity)]
    type DeleteDef = ServiceBaseBuild<
        <Behaviors as CRUDBehaviors>::OnDeleteIn,
        SeqB!(
            <BeforeFilter as CRUDHook>::Hook,
            Lift<<BeforeFilter as CRUDHook>::OnDelete, Self::Ctx>
        ),
        Convert<
            WithHookResult<
                <BeforeFilter as CRUDHook>::HookOut,
                <Behaviors as CRUDBehaviors>::OnDeleteIn,
            >,
            <<Behaviors as CRUDBehaviors>::OnDelete as Behavior>::In,
        >,
        Behaviors::OnDelete,
        Identity<<<Behaviors as CRUDBehaviors>::OnDelete as Behavior>::Out>,
        <<Behaviors as CRUDBehaviors>::OnDelete as Behavior>::Out,
    >;

    #[allow(clippy::type_complexity)]
    type FindOneDef = ServiceBaseBuild<
        <Behaviors as CRUDBehaviors>::OnFindOneIn,
        SeqB!(
            <BeforeFilter as CRUDHook>::Hook,
            Lift<<BeforeFilter as CRUDHook>::OnFindOne, Self::Ctx>
        ),
        Convert<
            WithHookResult<
                <BeforeFilter as CRUDHook>::HookOut,
                <Behaviors as CRUDBehaviors>::OnFindOneIn,
            >,
            <<Behaviors as CRUDBehaviors>::OnFindOne as Behavior>::In,
        >,
        Behaviors::OnFindOne,
        Convert<
            <<Behaviors as CRUDBehaviors>::OnFindOne as Behavior>::Out,
            <Behaviors as CRUDBehaviors>::OnFindOneOut,
        >,
        <Behaviors as CRUDBehaviors>::OnFindOneOut,
    >;

    #[allow(clippy::type_complexity)]
    type FindManyDef = ServiceBaseBuild<
        <Behaviors as CRUDBehaviors>::OnFindManyIn,
        SeqB!(
            <BeforeFilter as CRUDHook>::Hook,
            Lift<<BeforeFilter as CRUDHook>::OnFindMany, Self::Ctx>
        ),
        Convert<
            WithHookResult<
                <BeforeFilter as CRUDHook>::HookOut,
                <Behaviors as CRUDBehaviors>::OnFindManyIn,
            >,
            <<Behaviors as CRUDBehaviors>::OnFindMany as Behavior>::In,
        >,
        Behaviors::OnFindMany,
        Convert<
            <<Behaviors as CRUDBehaviors>::OnFindMany as Behavior>::Out,
            <Behaviors as CRUDBehaviors>::OnFindManyOut,
        >,
        <Behaviors as CRUDBehaviors>::OnFindManyOut,
    >;
}
