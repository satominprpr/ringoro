use async_trait::async_trait;
use std::marker::PhantomData;

use crate::behavior::Behavior;
use crate::core::Callable;
use crate::result::Result;

struct Lift<F, Ctx>
where
    F: Callable,
    F::In: Clone,
{
    f: F,
    p: PhantomData<fn() -> Ctx>,
}

#[async_trait(?Send)]
impl<F, Ctx> Behavior for Lift<F, Ctx>
where
    F: Callable,
    F::In: Clone,
{
    type In = F::In;
    type Out = F::Out;
    type Ctx = Ctx;

    async fn apply(i: &Self::In, _ctx: &Self::Ctx) -> Self {
        Self {
            f: F::apply(i.clone()),
            p: PhantomData,
        }
    }

    fn result(self) -> Result<Self::Out> {
        self.f.result()
    }
}

#[cfg(test)]
mod test_lift {
    use pretty_assertions::assert_eq;

    use crate::core::{RefCall, RefDef};

    use super::*;

    #[derive(Clone)]
    struct In(i8);

    #[derive(Debug, PartialEq)]
    struct Out(i8);
    struct Ctx();

    struct D {}
    impl RefDef for D {
        type In = In;
        type Out = Out;

        fn def(i: &In) -> Result<Out> {
            Ok(Out(i.0))
        }
    }

    #[tokio::test]
    async fn test_lift() {
        assert_eq!(
            Out(1),
            Lift::<RefCall<D>, Ctx>::apply(&In(1), &Ctx {})
                .await
                .result()
                .unwrap()
        );
    }
}
