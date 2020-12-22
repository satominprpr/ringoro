use std::marker::PhantomData;

use crate::core::Callable;
use ringoro_utils::{result::Result, simple_error};

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

pub struct Through<T>(T);

impl<T> Callable for Through<T> {
    type In = T;
    type Out = T;

    #[inline]
    fn apply(value: T) -> Self {
        Self(value)
    }

    #[inline]
    fn result(self) -> Result<T> {
        Ok(self.0)
    }
}

pub struct Deny<T>(PhantomData<T>);

impl<T> Callable for Deny<T> {
    type In = T;
    type Out = T;

    #[inline]
    fn apply(_: T) -> Self {
        Self(PhantomData)
    }

    #[inline]
    fn result(self) -> Result<T> {
        Err(simple_error!("Deny"))
    }
}

pub type Identity<T> = Through<T>;

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

    #[test]
    fn test_through() {
        assert_eq!(A(1), Through::<A>::apply(A(1)).result().unwrap())
    }

    #[test]
    fn test_deny() {
        assert_eq!(
            "Deny",
            format!("{}", Deny::<A>::apply(A(20)).result().unwrap_err())
        )
    }
}
