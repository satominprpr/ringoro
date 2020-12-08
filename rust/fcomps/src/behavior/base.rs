use async_trait::async_trait;

use crate::result::Result;

#[async_trait(?Send)]
pub trait Behavior {
    type In;
    type Out;
    type Ctx;

    async fn apply(input: &Self::In, ctx: &Self::Ctx) -> Self;
    fn result(self) -> Result<Self::Out>;
}

#[async_trait(?Send)]
pub trait EffectDef {
    type In;
    type Out;
    type Ctx;

    async fn def(input: &Self::In, ctx: &Self::Ctx) -> Result<Self::Out>;
}

pub struct Effect<Def>
where
    Def: EffectDef,
{
    result: Result<<Def as EffectDef>::Out>,
}

#[async_trait(?Send)]
impl<Def> Behavior for Effect<Def>
where
    Def: EffectDef,
{
    type In = <Def as EffectDef>::In;
    type Out = <Def as EffectDef>::Out;
    type Ctx = <Def as EffectDef>::Ctx;

    async fn apply(input: &Self::In, ctx: &Self::Ctx) -> Self {
        Self {
            result: Def::def(input, ctx).await,
        }
    }

    #[inline]
    fn result(self) -> Result<Self::Out> {
        match self.result {
            Ok(v) => Ok(v),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    struct In(i8);

    #[derive(Debug, PartialEq)]
    struct Out(i8, i8);
    struct Ctx(i8);

    struct EffectDefSuccess {}

    #[async_trait(?Send)]
    impl EffectDef for EffectDefSuccess {
        type In = In;
        type Out = Out;
        type Ctx = Ctx;
        async fn def(i: &In, c: &Ctx) -> Result<Out> {
            tokio::time::delay_for(std::time::Duration::from_millis(100)).await;
            Ok(Out(i.0 + 1, c.0))
        }
    }

    struct EffectDefFail {}

    #[async_trait(?Send)]
    impl EffectDef for EffectDefFail {
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
        let c = Ctx(3);
        let runner = Effect::<EffectDefSuccess>::apply(&i, &c).await;
        assert_eq!(Out(2, 3), runner.result().unwrap());
    }

    #[tokio::test]
    async fn test_effect_define_with_error() {
        let i = In(1);
        let c = Ctx(2);
        let runner = Effect::<EffectDefFail>::apply(&i, &c).await;
        assert_eq!("1, 2", format!("{}", runner.result().unwrap_err()));
    }
}
