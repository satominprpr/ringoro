use async_trait::async_trait;
use std::marker::PhantomData;

use crate::behavior::Behavior;
use crate::result::{Error, Result};

pub struct Composit<F, G>
where
    F: Behavior,
    G: Behavior<Out = <F as Behavior>::In, Ctx = <F as Behavior>::Ctx>,
{
    result: std::result::Result<F, Error>,
    p: PhantomData<G>,
}

#[async_trait(?Send)]
impl<F, G> Behavior for Composit<F, G>
where
    F: Behavior,
    G: Behavior<Out = <F as Behavior>::In, Ctx = <F as Behavior>::Ctx>,
{
    type In = <G as Behavior>::In;
    type Out = <F as Behavior>::Out;
    type Ctx = <F as Behavior>::Ctx;

    #[inline]
    async fn apply(input: Self::In, ctx: &Self::Ctx) -> Self {
        let g = G::apply(input, ctx).await;
        let result = match g.result() {
            Ok(r) => Ok(F::apply(r, ctx).await),
            Err(e) => Err(e),
        };
        Self {
            result,
            p: PhantomData,
        }
    }

    #[inline]
    fn result(self) -> Result<Self::Out> {
        match self.result {
            Ok(f) => f.result(),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::behavior::{Behave, BehaveDef};

    struct In(i8);
    struct Mid(i8, i8);

    #[derive(Debug, PartialEq)]
    struct Out(i8, i8, i8);
    struct Ctx(i8);

    struct DefFSuccess {}

    #[async_trait(?Send)]
    impl BehaveDef for DefFSuccess {
        type In = Mid;
        type Out = Out;
        type Ctx = Ctx;
        async fn def(i: Mid, c: &Ctx) -> Result<Out> {
            tokio::time::delay_for(std::time::Duration::from_millis(100)).await;
            Ok(Out(i.0, i.1, c.0))
        }
    }

    struct DefFFail {}

    #[async_trait(?Send)]
    impl BehaveDef for DefFFail {
        type In = Mid;
        type Out = Out;
        type Ctx = Ctx;
        async fn def(i: Mid, c: &Ctx) -> Result<Out> {
            use std::io::*;
            tokio::time::delay_for(std::time::Duration::from_millis(100)).await;
            Err(Error::new(ErrorKind::Other, format!("{}, {}, {}", i.0, i.1, c.0)).into())
        }
    }

    struct DefFPanic {}

    #[async_trait(?Send)]
    impl BehaveDef for DefFPanic {
        type In = Mid;
        type Out = Out;
        type Ctx = Ctx;
        async fn def(_: Mid, _: &Ctx) -> Result<Out> {
            tokio::time::delay_for(std::time::Duration::from_millis(100)).await;
            panic!()
        }
    }

    struct DefGSuccess {}

    #[async_trait(?Send)]
    impl BehaveDef for DefGSuccess {
        type In = In;
        type Out = Mid;
        type Ctx = Ctx;
        async fn def(i: In, c: &Ctx) -> Result<Mid> {
            tokio::time::delay_for(std::time::Duration::from_millis(100)).await;
            Ok(Mid(i.0, c.0))
        }
    }

    struct DefGFail {}

    #[async_trait(?Send)]
    impl BehaveDef for DefGFail {
        type In = In;
        type Out = Mid;
        type Ctx = Ctx;
        async fn def(i: In, c: &Ctx) -> Result<Mid> {
            use std::io::*;
            tokio::time::delay_for(std::time::Duration::from_millis(100)).await;
            Err(Error::new(ErrorKind::Other, format!("{}, {}", i.0, c.0)).into())
        }
    }

    type CompositSuccess = Composit<Behave<DefFSuccess>, Behave<DefGSuccess>>;

    #[tokio::test]
    async fn test_composit_successed() {
        let input = In(1);
        let ctx = Ctx(10);

        assert_eq!(
            Out(1, 10, 10),
            CompositSuccess::apply(input, &ctx).await.result().unwrap()
        );
    }

    type CompositFailOnF = Composit<Behave<DefFFail>, Behave<DefGSuccess>>;

    #[tokio::test]
    async fn test_composit_error_on_f() {
        let input = In(1);
        let ctx = Ctx(10);

        assert_eq!(
            "1, 10, 10",
            format!(
                "{}",
                CompositFailOnF::apply(input, &ctx)
                    .await
                    .result()
                    .unwrap_err()
            )
        );
    }

    type CompositFailOnG = Composit<Behave<DefFPanic>, Behave<DefGFail>>;

    #[tokio::test]
    async fn test_composit_error_on_g() {
        let input = In(1);
        let ctx = Ctx(10);

        assert_eq!(
            "1, 10",
            format!(
                "{}",
                CompositFailOnG::apply(input, &ctx)
                    .await
                    .result()
                    .unwrap_err()
            )
        );
    }
}
