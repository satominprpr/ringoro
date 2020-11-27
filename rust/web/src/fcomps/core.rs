use crate::result::{Error, Result};

pub trait Callable {
    type In;
    type Out;
    type Ctx;

    fn call(input: &Self::In, context: &Self::Ctx) -> Self;
    fn result(&self) -> Result<&Self::Out>;
}

pub trait Definer {
    type In;
    type Out;
    type Ctx;

    fn def(input: &Self::In, context: &Self::Ctx) -> Result<Self::Out>;
}

pub struct Effect<Def>
where
    Def: Definer,
{
    result: Result<<Def as Definer>::Out>,
}

impl<Def> Callable for Effect<Def>
where
    Def: Definer,
{
    type In = <Def as Definer>::In;
    type Out = <Def as Definer>::Out;
    type Ctx = <Def as Definer>::Ctx;

    #[inline]
    fn call(input: &Self::In, context: &Self::Ctx) -> Self {
        Self {
            result: Def::def(input, context),
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

    impl Definer for EffectDefSuccess {
        type In = In;
        type Out = Out;
        type Ctx = Ctx;
        fn def(i: &In, c: &Ctx) -> Result<Out> {
            Ok(Out(i.0, c.0))
        }
    }

    struct EffectDefFail {}

    impl Definer for EffectDefFail {
        type In = In;
        type Out = Out;
        type Ctx = Ctx;
        fn def(i: &In, c: &Ctx) -> Result<Out> {
            use std::io::*;
            Err(Error::new(ErrorKind::Other, format!("{}, {}", i.0, c.0)).into())
        }
    }

    #[test]
    fn test_effect_define_with_success() {
        let i = In(1);
        let c = Ctx(2);
        let runner = Effect::<EffectDefSuccess>::call(&i, &c);
        assert_eq!(Out(1, 2), *runner.result().unwrap());
    }

    #[test]
    fn test_effect_define_with_error() {
        let i = In(1);
        let c = Ctx(2);
        let runner = Effect::<EffectDefFail>::call(&i, &c);
        assert_eq!("1, 2", format!("{}", runner.result().unwrap_err()));
    }
}

pub struct Composit<F, G>
where
    F: Callable,
    G: Callable<Out = <F as Callable>::In, Ctx = <F as Callable>::Ctx>,
{
    result: std::result::Result<(F, G), Error>,
}

impl<F, G> Callable for Composit<F, G>
where
    F: Callable,
    G: Callable<Out = <F as Callable>::In, Ctx = <F as Callable>::Ctx>,
{
    type In = <G as Callable>::In;
    type Out = <F as Callable>::Out;
    type Ctx = <F as Callable>::Ctx;

    #[inline]
    fn call(input: &Self::In, context: &Self::Ctx) -> Self {
        let g = G::call(input, context);
        let result = match g.result() {
            Ok(r) => Ok((F::call(r, context), g)),
            Err(e) => Err(e),
        };
        Self { result }
    }

    #[inline]
    fn result(&self) -> Result<&Self::Out> {
        match &self.result {
            Ok((f, _)) => f.result(),
            Err(e) => Err(e.clone()),
        }
    }
}

#[cfg(test)]
mod comosit_test {
    use super::*;
    use pretty_assertions::assert_eq;

    struct In(i8);
    struct Mid(i8, i8);

    #[derive(Debug, PartialEq)]
    struct Out(i8, i8, i8);
    struct Ctx(i8);

    struct DefFSuccess {}

    impl Definer for DefFSuccess {
        type In = Mid;
        type Out = Out;
        type Ctx = Ctx;
        fn def(i: &Mid, c: &Ctx) -> Result<Out> {
            Ok(Out(i.0, i.1, c.0))
        }
    }

    struct DefFFail {}

    impl Definer for DefFFail {
        type In = Mid;
        type Out = Out;
        type Ctx = Ctx;
        fn def(i: &Mid, c: &Ctx) -> Result<Out> {
            use std::io::*;
            Err(Error::new(ErrorKind::Other, format!("{}, {}, {}", i.0, i.1, c.0)).into())
        }
    }

    struct DefFPanic {}

    impl Definer for DefFPanic {
        type In = Mid;
        type Out = Out;
        type Ctx = Ctx;
        fn def(_: &Mid, _: &Ctx) -> Result<Out> {
            panic!()
        }
    }

    struct DefGSuccess {}

    impl Definer for DefGSuccess {
        type In = In;
        type Out = Mid;
        type Ctx = Ctx;
        fn def(i: &In, c: &Ctx) -> Result<Mid> {
            Ok(Mid(i.0, c.0))
        }
    }

    struct DefGFail {}

    impl Definer for DefGFail {
        type In = In;
        type Out = Mid;
        type Ctx = Ctx;
        fn def(i: &In, c: &Ctx) -> Result<Mid> {
            use std::io::*;
            Err(Error::new(ErrorKind::Other, format!("{}, {}", i.0, c.0)).into())
        }
    }

    type CompositSuccess = Composit<Effect<DefFSuccess>, Effect<DefGSuccess>>;

    #[test]
    fn test_composit_successed() {
        let input = In(1);
        let ctx = Ctx(10);

        assert_eq!(
            Out(1, 10, 10),
            *CompositSuccess::call(&input, &ctx).result().unwrap()
        );
    }

    type CompositFailOnF = Composit<Effect<DefFFail>, Effect<DefGSuccess>>;

    #[test]
    fn test_composit_error_on_f() {
        let input = In(1);
        let ctx = Ctx(10);

        assert_eq!(
            "1, 10, 10",
            format!(
                "{}",
                *CompositFailOnF::call(&input, &ctx).result().unwrap_err()
            )
        );
    }

    type CompositFailOnG = Composit<Effect<DefFPanic>, Effect<DefGFail>>;

    #[test]
    fn test_composit_error_on_g() {
        let input = In(1);
        let ctx = Ctx(10);

        assert_eq!(
            "1, 10",
            format!(
                "{}",
                *CompositFailOnG::call(&input, &ctx).result().unwrap_err()
            )
        );
    }
}
