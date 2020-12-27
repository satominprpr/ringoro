use async_trait::async_trait;
use std::marker::PhantomData;

use crate::{
    behavior::Behavior,
    core::{validate::Identity, Callable, Panic},
    result::Result,
};

pub struct Lift<F, Ctx>
where
    F: Callable,
{
    f: F,
    p: PhantomData<fn() -> Ctx>,
}

#[async_trait(?Send)]
impl<F, Ctx> Behavior for Lift<F, Ctx>
where
    F: Callable,
{
    type In = F::In;
    type Out = F::Out;
    type Ctx = Ctx;

    #[inline]
    async fn apply(i: Self::In, _ctx: &Self::Ctx) -> Self {
        Self {
            f: F::apply(i),
            p: PhantomData,
        }
    }

    #[inline]
    fn result(self) -> Result<Self::Out> {
        self.f.result()
    }
}

pub type NoBehave<T, Ctx> = Lift<Identity<T>, Ctx>;
pub type PanicBehave<In, Out, Ctx> = Lift<Panic<In, Out>, Ctx>;

#[cfg(test)]
mod test_lift {
    use pretty_assertions::assert_eq;

    use crate::core::{Call, Def};

    use super::*;

    #[derive(Clone)]
    struct In(i8);

    #[derive(Debug, PartialEq)]
    struct Out(i8);
    struct Ctx();

    struct D {}
    impl Def for D {
        type In = In;
        type Out = Out;

        fn def(i: In) -> Result<Out> {
            Ok(Out(i.0))
        }
    }

    #[tokio::test]
    async fn test_lift() {
        assert_eq!(
            Out(1),
            Lift::<Call<D>, Ctx>::apply(In(1), &Ctx {})
                .await
                .result()
                .unwrap()
        );
    }
}
