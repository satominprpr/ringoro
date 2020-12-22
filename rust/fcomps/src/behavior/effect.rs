use async_trait::async_trait;
use std::marker::PhantomData;

use crate::{behavior::Behavior, result::Result, Functor};

#[async_trait(?Send)]
pub trait Effector {
    type In;
    type Out;
    type Ctx;

    async fn def(input: &Self::In, ctx: &Self::Ctx) -> Result<Self::Out>;
}

pub struct Effect<Def>
where
    Def: Effector,
{
    input: Def::In,
    result: Result<Def::Out>,
}

#[async_trait(?Send)]
impl<Def> Behavior for Effect<Def>
where
    Def: Effector,
{
    type In = Def::In;
    type Out = Def::In;
    type Ctx = Def::Ctx;

    #[inline]
    async fn apply(input: Self::In, ctx: &Self::Ctx) -> Self {
        Self {
            result: Def::def(&input, ctx).await,
            input,
        }
    }

    #[inline]
    fn result(self) -> Result<Self::Out> {
        match self.result {
            Ok(_) => Ok(self.input),
            Err(err) => Err(err),
        }
    }
}

pub struct CloneEffector<T, Ctx>
where
    T: Clone,
{
    f: PhantomData<fn() -> (T, Ctx)>,
}

#[async_trait(?Send)]
impl<T, Ctx> Effector for CloneEffector<T, Ctx>
where
    T: Clone,
{
    type In = T;
    type Out = T;
    type Ctx = Ctx;

    #[inline]
    async fn def(input: &Self::In, _ctx: &Self::Ctx) -> Result<Self::Out> {
        Ok(input.clone())
    }
}

pub struct EffectorComposit<F, E>
where
    E: Effector,
    F: Behavior<In = <E as Effector>::Out, Ctx = <E as Effector>::Ctx>,
{
    p: PhantomData<fn() -> (F, E)>,
}

#[async_trait(?Send)]
impl<F, E> Effector for EffectorComposit<F, E>
where
    E: Effector,
    F: Behavior<In = <E as Effector>::Out, Ctx = <E as Effector>::Ctx>,
{
    type In = E::In;
    type Out = F::Out;
    type Ctx = E::Ctx;

    #[inline]
    async fn def(input: &Self::In, ctx: &Self::Ctx) -> Result<Self::Out> {
        match E::def(input, ctx).await {
            Ok(i) => F::apply(i, ctx).await.result(),
            Err(e) => Err(e),
        }
    }
}

pub struct CompositWithCloneFn<T, F>
where
    T: Clone,
    F: Behavior<In = T>,
{
    p: PhantomData<fn() -> (T, F)>,
}

impl<T, F> Functor for CompositWithCloneFn<T, F>
where
    T: Clone,
    F: Behavior<In = T>,
{
    type Result = EffectorComposit<F, CloneEffector<T, F::Ctx>>;
}

pub type CompositWithClone<T, F> = <CompositWithCloneFn<T, F> as Functor>::Result;
