use async_trait::async_trait;

use crate::result::Result;

#[async_trait(?Send)]
pub trait Callable {
    type In;
    type Out;
    type Ctx;

    async fn call(input: &Self::In, context: &Self::Ctx) -> Self;
    fn result(&self) -> Result<&Self::Out>;
}

#[async_trait(?Send)]
pub trait Definer {
    type In;
    type Out;
    type Ctx;

    async fn def(input: &Self::In, context: &Self::Ctx) -> Result<Self::Out>;
}

pub struct Effect<Def>
where
    Def: Definer,
{
    result: Result<<Def as Definer>::Out>,
}

#[async_trait(?Send)]
impl<Def> Callable for Effect<Def>
where
    Def: Definer,
{
    type In = <Def as Definer>::In;
    type Out = <Def as Definer>::Out;
    type Ctx = <Def as Definer>::Ctx;

    async fn call(input: &Self::In, context: &Self::Ctx) -> Self {
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
mod effect_test {
    use super::*;
    use pretty_assertions::assert_eq;

    struct In(i8);

    #[derive(Debug, PartialEq)]
    struct Out(i8, i8);
    struct Ctx(i8);

    struct EffectDefSuccess {}

    #[async_trait(?Send)]
    impl Definer for EffectDefSuccess {
        type In = In;
        type Out = Out;
        type Ctx = Ctx;
        async fn def(i: &In, c: &Ctx) -> Result<Out> {
            tokio::time::delay_for(std::time::Duration::from_millis(100)).await;
            Ok(Out(i.0, c.0))
        }
    }

    struct EffectDefFail {}

    #[async_trait(?Send)]
    impl Definer for EffectDefFail {
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
        let runner = Effect::<EffectDefSuccess>::call(&i, &c).await;
        assert_eq!(Out(1, 2), *runner.result().unwrap());
    }

    #[tokio::test]
    async fn test_effect_define_with_error() {
        let i = In(1);
        let c = Ctx(2);
        let runner = Effect::<EffectDefFail>::call(&i, &c).await;
        assert_eq!("1, 2", format!("{}", runner.result().unwrap_err()));
    }
}
