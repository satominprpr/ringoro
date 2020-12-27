use std::marker::PhantomData;

use async_trait::async_trait;

use crate::{
    behavior::{lift::Lift, Behave, BehaveDef, Behavior},
    convert::Convertible,
    result::Result,
    Callable, SeqB,
};

pub trait FromHookResult<HookOut, T>
where
    Self: Sized,
{
    fn from_hook_result(h: HookOut, t: T) -> Result<Self>;
}

pub trait HookResult {}

pub struct WithHookResult<H, T>(pub H, pub T);

impl<H, T, To> Convertible<To> for WithHookResult<H, T>
where
    To: FromHookResult<H, T>,
    H: HookResult,
{
    #[inline]
    fn convert(self) -> Result<To> {
        To::from_hook_result(self.0, self.1)
    }
}

impl<T, U> Convertible<T> for WithHookResult<(), U>
where
    U: Convertible<T>,
{
    fn convert(self) -> Result<T> {
        Ok(self.1.convert()?)
    }
}

impl<H> FromHookResult<H, ()> for ()
where
    H: HookResult,
{
    fn from_hook_result(_: H, _: ()) -> Result<()> {
        Ok(())
    }
}

pub trait ServiceBaseDef {
    type In;
    type FilterOut;
    type ServiceIn;
    type ServiceOut;
    type Out;
    type BeforeFilter: Behavior<In = (), Out = Self::FilterOut, Ctx = Self::Ctx>;
    type Converter: Callable<In = WithHookResult<Self::FilterOut, Self::In>, Out = Self::ServiceIn>;
    type ServiceBehavior: Behavior<In = Self::ServiceIn, Out = Self::ServiceOut, Ctx = Self::Ctx>;
    type AfterConverter: Callable<In = Self::ServiceOut, Out = Self::Out>;
    type Ctx;
}

pub struct BeforeFilterDef<In, Filter>
where
    Filter: Behavior<In = ()>,
{
    p: PhantomData<fn() -> (In, Filter)>,
}

#[async_trait(?Send)]
impl<In, Filter> BehaveDef for BeforeFilterDef<In, Filter>
where
    Filter: Behavior<In = ()>,
{
    type In = In;
    type Out = WithHookResult<Filter::Out, In>;
    type Ctx = Filter::Ctx;

    #[inline]
    async fn def(input: Self::In, ctx: &Self::Ctx) -> Result<Self::Out> {
        let filtered = Filter::apply((), ctx).await;
        let result = filtered.result()?;
        Ok(WithHookResult(result, input))
    }
}

pub type ServiceBase<Def> = SeqB!(
        Behave<BeforeFilterDef<<Def as ServiceBaseDef>::In, <Def as ServiceBaseDef>::BeforeFilter>>,
        Lift<<Def as ServiceBaseDef>::Converter,
        <Def as ServiceBaseDef>::Ctx>, <Def as ServiceBaseDef>::ServiceBehavior, Lift<<Def as ServiceBaseDef>::AfterConverter, <Def as ServiceBaseDef>::Ctx>);

pub struct ServiceBaseBuild<In, BeforeFilter, Converter, ServiceBehavior, AfterConverter, Out>
where
    BeforeFilter: Behavior<In = (), Ctx = ServiceBehavior::Ctx>,
    Converter: Callable<In = WithHookResult<BeforeFilter::Out, In>, Out = ServiceBehavior::In>,
    AfterConverter: Callable<In = ServiceBehavior::Out, Out = Out>,
    ServiceBehavior: Behavior,
{
    #[allow(clippy::type_complexity)]
    p: PhantomData<fn() -> (BeforeFilter, Converter, ServiceBehavior, AfterConverter)>,
}

impl<In, BeforeFilter, Converter, ServiceBehavior, AfterConverter, Out> ServiceBaseDef
    for ServiceBaseBuild<In, BeforeFilter, Converter, ServiceBehavior, AfterConverter, Out>
where
    BeforeFilter: Behavior<In = (), Ctx = ServiceBehavior::Ctx>,
    Converter: Callable<In = WithHookResult<BeforeFilter::Out, In>, Out = ServiceBehavior::In>,
    AfterConverter: Callable<In = ServiceBehavior::Out, Out = Out>,
    ServiceBehavior: Behavior,
{
    type In = In;
    type FilterOut = BeforeFilter::Out;
    type ServiceIn = ServiceBehavior::In;
    type ServiceOut = ServiceBehavior::Out;
    type Out = AfterConverter::Out;
    type BeforeFilter = BeforeFilter;
    type Converter = Converter;
    type ServiceBehavior = ServiceBehavior;
    type AfterConverter = AfterConverter;
    type Ctx = BeforeFilter::Ctx;
}
