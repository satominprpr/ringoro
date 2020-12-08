use std::marker::PhantomData;

use crate::core::Callable;
use ringoro_utils::result::Result;

pub trait ValidateRefDef {
    type T;

    fn def(input: &Self::T) -> Result<()>;
}

pub struct Validate<D>
where
    D: ValidateRefDef,
{
    result: Result<D::T>,
    p: PhantomData<D>,
}

impl<D> Callable for Validate<D>
where
    D: ValidateRefDef,
{
    type In = D::T;
    type Out = D::T;

    #[inline]
    fn apply(input: Self::In) -> Self {
        Self {
            result: match D::def(&input) {
                Ok(()) => Ok(input),
                Err(e) => Err(e),
            },
            p: PhantomData,
        }
    }

    #[inline]
    fn result(self) -> Result<Self::Out> {
        self.result
    }
}

#[cfg(test)]
mod test_validate {
    use pretty_assertions::assert_eq;
    use ringoro_utils::simple_error;

    use super::*;

    #[derive(Debug, PartialEq)]
    struct A(i8);

    struct Define {}

    impl ValidateRefDef for Define {
        type T = A;
        fn def(i: &A) -> Result<()> {
            let val = i.0;
            if val < 10 {
                Ok(())
            } else {
                Err(simple_error!("{}", val))
            }
        }
    }

    #[test]
    fn test_validate_success() {
        assert_eq!(A(1), Validate::<Define>::apply(A(1)).result().unwrap())
    }

    #[test]
    fn test_validat_fail() {
        assert_eq!(
            "20",
            format!("{}", Validate::<Define>::apply(A(20)).result().unwrap_err())
        )
    }
}
