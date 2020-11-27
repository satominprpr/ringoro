use std::marker::PhantomData;

use crate::result::{Error, Result};

pub trait Callable<In, Out, Ctx> {
    fn call(input: &In, context: &Ctx) -> Self;
    fn result(&self) -> Result<&Out>;
}

pub trait Definer<In, Out, Ctx> {
    fn def(input: &In, context: &Ctx) -> Result<Out>;
}

pub struct Composit<F, G, In, Mid, Out, Ctx>
where
    F: Callable<Mid, Out, Ctx>,
    G: Callable<In, Mid, Ctx>,
{
    result: std::result::Result<(F, G), Error>,
    #[allow(clippy::type_complexity)]
    p: PhantomData<fn() -> (In, Mid, Out, Ctx)>,
}

impl<F, G, In, Mid, Out, Ctx> Callable<In, Out, Ctx> for Composit<F, G, In, Mid, Out, Ctx>
where
    F: Callable<Mid, Out, Ctx>,
    G: Callable<In, Mid, Ctx>,
{
    #[inline]
    fn call(input: &In, context: &Ctx) -> Self {
        let g = G::call(input, context);
        let result = match g.result() {
            Ok(r) => Ok((F::call(r, context), g)),
            Err(e) => Err(e),
        };
        Self {
            result,
            p: PhantomData,
        }
    }

    #[inline]
    fn result(&self) -> Result<&Out> {
        match &self.result {
            Ok((f, _)) => f.result(),
            Err(e) => Err(e.clone()),
        }
    }
}

pub struct Effect<Def, In, Out, Ctx>
where
    Def: Definer<In, Out, Ctx>,
{
    result: Result<Out>,
    #[allow(clippy::type_complexity)]
    p: PhantomData<fn() -> (In, Def, Ctx)>,
}

impl<Def, In, Out, Ctx> Callable<In, Out, Ctx> for Effect<Def, In, Out, Ctx>
where
    Def: Definer<In, Out, Ctx>,
{
    #[inline]
    fn call(input: &In, context: &Ctx) -> Self {
        Self {
            result: Def::def(input, context),
            p: PhantomData,
        }
    }

    #[inline]
    fn result(&self) -> Result<&Out> {
        match &self.result {
            Ok(v) => Ok(v),
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

    impl Definer<Mid, Out, Ctx> for DefFSuccess {
        fn def(i: &Mid, c: &Ctx) -> Result<Out> {
            Ok(Out(i.0, i.1, c.0))
        }
    }

    struct DefFFail {}

    impl Definer<Mid, Out, Ctx> for DefFFail {
        fn def(i: &Mid, c: &Ctx) -> Result<Out> {
            use std::io::*;
            Err(Error::new(ErrorKind::Other, format!("{}, {}, {}", i.0, i.1, c.0)).into())
        }
    }

    struct DefFPanic {}

    impl Definer<Mid, Out, Ctx> for DefFPanic {
        fn def(_: &Mid, _: &Ctx) -> Result<Out> {
            panic!()
        }
    }

    struct DefGSuccess {}

    impl Definer<In, Mid, Ctx> for DefGSuccess {
        fn def(i: &In, c: &Ctx) -> Result<Mid> {
            Ok(Mid(i.0, c.0))
        }
    }

    struct DefGFail {}

    impl Definer<In, Mid, Ctx> for DefGFail {
        fn def(i: &In, c: &Ctx) -> Result<Mid> {
            use std::io::*;
            Err(Error::new(ErrorKind::Other, format!("{}, {}", i.0, c.0)).into())
        }
    }

    type CompositSuccess = Composit<
        Effect<DefFSuccess, Mid, Out, Ctx>,
        Effect<DefGSuccess, In, Mid, Ctx>,
        In,
        Mid,
        Out,
        Ctx,
    >;

    #[test]
    fn test_composit_successed() {
        let input = In(1);
        let ctx = Ctx(10);

        assert_eq!(
            Out(1, 10, 10),
            *CompositSuccess::call(&input, &ctx).result().unwrap()
        );
    }

    type CompositFailOnF = Composit<
        Effect<DefFFail, Mid, Out, Ctx>,
        Effect<DefGSuccess, In, Mid, Ctx>,
        In,
        Mid,
        Out,
        Ctx,
    >;

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

    type CompositFailOnG = Composit<
        Effect<DefFPanic, Mid, Out, Ctx>,
        Effect<DefGFail, In, Mid, Ctx>,
        In,
        Mid,
        Out,
        Ctx,
    >;

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

#[cfg(test)]
mod effect_test {
    use super::*;
    use pretty_assertions::assert_eq;

    pub fn effect<Def, In, Out, Ctx>(
        _def: Def,
        input: &In,
        context: &Ctx,
    ) -> Effect<Def, In, Out, Ctx>
    where
        Def: Definer<In, Out, Ctx>,
    {
        Effect::<Def, In, Out, Ctx>::call(input, context)
    }

    struct In(i8);

    #[derive(Debug, PartialEq)]
    struct Out(i8, i8);
    struct Ctx(i8);

    struct EffectDefSuccess {}

    impl Definer<In, Out, Ctx> for EffectDefSuccess {
        fn def(i: &In, c: &Ctx) -> Result<Out> {
            Ok(Out(i.0, c.0))
        }
    }

    struct EffectDefFail {}

    impl Definer<In, Out, Ctx> for EffectDefFail {
        fn def(i: &In, c: &Ctx) -> Result<Out> {
            use std::io::*;
            Err(Error::new(ErrorKind::Other, format!("{}, {}", i.0, c.0)).into())
        }
    }

    #[test]
    fn test_effect_define_with_success() {
        let i = In(1);
        let c = Ctx(2);
        let runner = effect(EffectDefSuccess {}, &i, &c);
        assert_eq!(Out(1, 2), *runner.result().unwrap());
    }

    #[test]
    fn test_effect_define_with_error() {
        let i = In(1);
        let c = Ctx(2);
        let runner = effect(EffectDefFail {}, &i, &c);
        assert_eq!("1, 2", format!("{}", runner.result().unwrap_err()));
    }
}
