use std::marker::PhantomData;

use crate::core::Callable;
use crate::result::{Error, Result};

pub struct Composit<F, G>
where
    F: Callable,
    G: Callable<Out = F::In>,
{
    result: std::result::Result<F, Error>,
    p: PhantomData<fn() -> G>,
}

impl<F, G> Callable for Composit<F, G>
where
    F: Callable,
    G: Callable<Out = F::In>,
{
    type In = G::In;
    type Out = F::Out;

    #[inline]
    fn apply(input: Self::In) -> Self {
        let g = G::apply(input);
        let result = match g.result() {
            Ok(r) => Ok(F::apply(r)),
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
    use crate::core::{Call, Def};

    struct In(i8);
    struct Mid(i8);

    #[derive(Debug, PartialEq)]
    struct Out(i8);

    struct DefFSuccess {}

    impl Def for DefFSuccess {
        type In = Mid;
        type Out = Out;
        fn def(i: Mid) -> Result<Out> {
            Ok(Out(i.0 + 1))
        }
    }

    struct DefFFail {}

    impl Def for DefFFail {
        type In = Mid;
        type Out = Out;
        fn def(i: Mid) -> Result<Out> {
            use std::io::*;
            Err(Error::new(ErrorKind::Other, format!("{}", i.0 + 1)).into())
        }
    }

    struct DefFPanic {}

    impl Def for DefFPanic {
        type In = Mid;
        type Out = Out;
        fn def(_: Mid) -> Result<Out> {
            panic!()
        }
    }

    struct DefGSuccess {}

    impl Def for DefGSuccess {
        type In = In;
        type Out = Mid;
        fn def(i: In) -> Result<Mid> {
            Ok(Mid(i.0 + 2))
        }
    }

    struct DefGFail {}

    impl Def for DefGFail {
        type In = In;
        type Out = Mid;
        fn def(i: In) -> Result<Mid> {
            use std::io::*;
            Err(Error::new(ErrorKind::Other, format!("{}", i.0 + 2)).into())
        }
    }

    type CompositSuccess = Composit<Call<DefFSuccess>, Call<DefGSuccess>>;

    #[test]
    fn test_composit_successed() {
        let input = In(1);

        assert_eq!(Out(4), CompositSuccess::apply(input).result().unwrap());
    }

    type CompositFailOnF = Composit<Call<DefFFail>, Call<DefGSuccess>>;

    #[test]
    fn test_composit_error_on_f() {
        let input = In(1);

        assert_eq!(
            "4",
            format!("{}", CompositFailOnF::apply(input).result().unwrap_err())
        );
    }

    type CompositFailOnG = Composit<Call<DefFPanic>, Call<DefGFail>>;

    #[test]
    fn test_composit_error_on_g() {
        let input = In(1);

        assert_eq!(
            "3",
            format!("{}", CompositFailOnG::apply(input).result().unwrap_err())
        );
    }
}
