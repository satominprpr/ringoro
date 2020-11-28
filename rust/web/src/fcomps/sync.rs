use async_trait::async_trait;

use crate::result::{Error, Result};

#[async_trait(?Send)]
pub trait ACallable {
    type In;
    type Out;
    type Ctx;

    async fn apply(input: &Self::In, context: &Self::Ctx) -> Self;
    fn result(&self) -> Result<&Self::Out>;
}

#[async_trait(?Send)]
pub trait ADefiner {
    type In;
    type Out;
    type Ctx;

    async fn def(input: &Self::In, context: &Self::Ctx) -> Result<Self::Out>;
}

pub struct AEffect<Def>
where
    Def: ADefiner,
{
    result: Result<<Def as ADefiner>::Out>,
}

#[async_trait(?Send)]
impl<Def> ACallable for AEffect<Def>
where
    Def: ADefiner,
{
    type In = <Def as ADefiner>::In;
    type Out = <Def as ADefiner>::Out;
    type Ctx = <Def as ADefiner>::Ctx;

    async fn apply(input: &Self::In, context: &Self::Ctx) -> Self {
        Self {
            result: Def::def(input, context).await,
        }
    }

    #[inline]
    fn result(&self) -> Result<&Self::Out> {
        match &self.result {
            Ok(v) => Ok(v),
            Err(e) => Err(e.clone()),
        }
    }
}

#[cfg(test)]
mod aeffect_test {
    use super::*;
    use pretty_assertions::assert_eq;

    struct In(i8);

    #[derive(Debug, PartialEq)]
    struct Out(i8, i8);
    struct Ctx(i8);

    struct AEffectDefSuccess {}

    #[async_trait(?Send)]
    impl ADefiner for AEffectDefSuccess {
        type In = In;
        type Out = Out;
        type Ctx = Ctx;
        async fn def(i: &In, c: &Ctx) -> Result<Out> {
            tokio::time::delay_for(std::time::Duration::from_millis(100)).await;
            Ok(Out(i.0, c.0))
        }
    }

    struct AEffectDefFail {}

    #[async_trait(?Send)]
    impl ADefiner for AEffectDefFail {
        type In = In;
        type Out = Out;
        type Ctx = Ctx;
        async fn def(i: &In, c: &Ctx) -> Result<Out> {
            use std::io::*;
            tokio::time::delay_for(std::time::Duration::from_millis(100)).await;
            Err(Error::new(ErrorKind::Other, format!("{}, {}", i.0, c.0)).into())
        }
    }

    #[tokio::test]
    async fn test_effect_define_with_success() {
        let i = In(1);
        let c = Ctx(2);
        let runner = AEffect::<AEffectDefSuccess>::apply(&i, &c).await;
        assert_eq!(Out(1, 2), *runner.result().unwrap());
    }

    #[tokio::test]
    async fn test_effect_define_with_error() {
        let i = In(1);
        let c = Ctx(2);
        let runner = AEffect::<AEffectDefFail>::apply(&i, &c).await;
        assert_eq!("1, 2", format!("{}", runner.result().unwrap_err()));
    }
}

pub struct AComposit<F, G>
where
    F: ACallable,
    G: ACallable<Out = <F as ACallable>::In, Ctx = <F as ACallable>::Ctx>,
{
    result: std::result::Result<(F, G), Error>,
}

#[async_trait(?Send)]
impl<F, G> ACallable for AComposit<F, G>
where
    F: ACallable,
    G: ACallable<Out = <F as ACallable>::In, Ctx = <F as ACallable>::Ctx>,
{
    type In = <G as ACallable>::In;
    type Out = <F as ACallable>::Out;
    type Ctx = <F as ACallable>::Ctx;

    async fn apply(input: &Self::In, context: &Self::Ctx) -> Self {
        let g = G::apply(input, context).await;
        let result = match g.result() {
            Ok(r) => Ok((F::apply(r, context).await, g)),
            Err(e) => Err(e),
        };
        Self { result }
    }

    fn result(&self) -> Result<&Self::Out> {
        match &self.result {
            Ok((f, _)) => f.result(),
            Err(e) => Err(e.clone()),
        }
    }
}

#[cfg(test)]
mod acomosit_test {
    use super::*;
    use pretty_assertions::assert_eq;

    struct In(i8);
    struct Mid(i8, i8);

    #[derive(Debug, PartialEq)]
    struct Out(i8, i8, i8);
    struct Ctx(i8);

    struct DefFSuccess {}

    #[async_trait(?Send)]
    impl ADefiner for DefFSuccess {
        type In = Mid;
        type Out = Out;
        type Ctx = Ctx;
        async fn def(i: &Mid, c: &Ctx) -> Result<Out> {
            tokio::time::delay_for(std::time::Duration::from_millis(100)).await;
            Ok(Out(i.0, i.1, c.0))
        }
    }

    struct DefFFail {}

    #[async_trait(?Send)]
    impl ADefiner for DefFFail {
        type In = Mid;
        type Out = Out;
        type Ctx = Ctx;
        async fn def(i: &Mid, c: &Ctx) -> Result<Out> {
            use std::io::*;
            tokio::time::delay_for(std::time::Duration::from_millis(100)).await;
            Err(Error::new(ErrorKind::Other, format!("{}, {}, {}", i.0, i.1, c.0)).into())
        }
    }

    struct DefFPanic {}

    #[async_trait(?Send)]
    impl ADefiner for DefFPanic {
        type In = Mid;
        type Out = Out;
        type Ctx = Ctx;
        async fn def(_: &Mid, _: &Ctx) -> Result<Out> {
            tokio::time::delay_for(std::time::Duration::from_millis(100)).await;
            panic!()
        }
    }

    struct DefGSuccess {}

    #[async_trait(?Send)]
    impl ADefiner for DefGSuccess {
        type In = In;
        type Out = Mid;
        type Ctx = Ctx;
        async fn def(i: &In, c: &Ctx) -> Result<Mid> {
            tokio::time::delay_for(std::time::Duration::from_millis(100)).await;
            Ok(Mid(i.0, c.0))
        }
    }

    struct DefGFail {}

    #[async_trait(?Send)]
    impl ADefiner for DefGFail {
        type In = In;
        type Out = Mid;
        type Ctx = Ctx;
        async fn def(i: &In, c: &Ctx) -> Result<Mid> {
            use std::io::*;
            tokio::time::delay_for(std::time::Duration::from_millis(100)).await;
            Err(Error::new(ErrorKind::Other, format!("{}, {}", i.0, c.0)).into())
        }
    }

    type ACompositSuccess = AComposit<AEffect<DefFSuccess>, AEffect<DefGSuccess>>;

    #[tokio::test]
    async fn test_composit_successed() {
        let input = In(1);
        let ctx = Ctx(10);

        assert_eq!(
            Out(1, 10, 10),
            *ACompositSuccess::apply(&input, &ctx)
                .await
                .result()
                .unwrap()
        );
    }

    type ACompositFailOnF = AComposit<AEffect<DefFFail>, AEffect<DefGSuccess>>;

    #[tokio::test]
    async fn test_composit_error_on_f() {
        let input = In(1);
        let ctx = Ctx(10);

        assert_eq!(
            "1, 10, 10",
            format!(
                "{}",
                *ACompositFailOnF::apply(&input, &ctx)
                    .await
                    .result()
                    .unwrap_err()
            )
        );
    }

    type ACompositFailOnG = AComposit<AEffect<DefFPanic>, AEffect<DefGFail>>;

    #[tokio::test]
    async fn test_composit_error_on_g() {
        let input = In(1);
        let ctx = Ctx(10);

        assert_eq!(
            "1, 10",
            format!(
                "{}",
                *ACompositFailOnG::apply(&input, &ctx)
                    .await
                    .result()
                    .unwrap_err()
            )
        );
    }
}

use crate::fcomps::core::Callable;

struct Lift<F>
where
    F: Callable,
{
    f: F,
}

#[async_trait(?Send)]
impl<F> ACallable for Lift<F>
where
    F: Callable,
{
    type In = <F as Callable>::In;
    type Out = <F as Callable>::Out;
    type Ctx = <F as Callable>::Ctx;

    async fn apply(i: &Self::In, ctx: &Self::Ctx) -> Self {
        Self {
            f: F::apply(i, ctx),
        }
    }

    fn result(&self) -> Result<&Self::Out> {
        self.f.result()
    }
}

#[cfg(test)]
mod test_lift {
    use pretty_assertions::assert_eq;

    use crate::fcomps::core::{Definer, Effect};
    use crate::result::Result;

    use super::{ACallable, Lift};

    type In = i8;
    type Out = (i8, i8);
    type Ctx = i8;

    struct D {}
    impl Definer for D {
        type In = In;
        type Out = Out;
        type Ctx = Ctx;

        fn def(i: &In, ctx: &Ctx) -> Result<Out> {
            Ok((*i, *ctx))
        }
    }

    #[tokio::test]
    async fn test_lift() {
        assert_eq!(
            (1i8, 2i8),
            *Lift::<Effect<D>>::apply(&1i8, &2i8).await.result().unwrap()
        );
    }
}
