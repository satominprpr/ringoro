use std::any::type_name;
use std::convert::TryFrom;
use std::marker::PhantomData;

use crate::core::{Call, Callable, Def, FcompError, RefCall, RefDef};
use crate::result::{raise, Result};

pub struct DefWithTryFrom<From, To>
where
    To: TryFrom<From>,
{
    p: PhantomData<fn() -> (From, To)>,
}

impl<From, To> Def for DefWithTryFrom<From, To>
where
    To: TryFrom<From>,
{
    type In = From;
    type Out = To;

    #[inline]
    fn def(input: From) -> Result<To> {
        match TryFrom::try_from(input) {
            Ok(v) => Ok(v),
            Err(_) => raise(FcompError::ConvertType {
                from: String::from(type_name::<From>()),
                to: String::from(type_name::<To>()),
            }),
        }
    }
}

pub type WithTryFrom<From, To> = Call<DefWithTryFrom<From, To>>;

pub struct DefWithTryFromRef<From, To>
where
    for<'a> To: TryFrom<&'a From>,
{
    p: PhantomData<fn() -> (From, To)>,
}

impl<From, To> RefDef for DefWithTryFromRef<From, To>
where
    for<'a> To: TryFrom<&'a From>,
{
    type In = From;
    type Out = To;

    #[inline]
    fn def(input: &From) -> Result<To> {
        match TryFrom::try_from(input) {
            Ok(v) => Ok(v),
            Err(_) => raise(FcompError::ConvertType {
                from: String::from(type_name::<From>()),
                to: String::from(type_name::<To>()),
            }),
        }
    }
}

pub type WithTryFromRef<From, To> = RefCall<DefWithTryFromRef<From, To>>;

pub trait OptionMapRefDef {
    type T;

    fn def(input: &Self::T) -> Result<Option<Self::T>>;
}

pub struct OptionMap<D>
where
    D: OptionMapRefDef,
{
    result: Result<Option<D::T>>,
    input: D::T,
}

impl<D> Callable for OptionMap<D>
where
    D: OptionMapRefDef,
{
    type In = D::T;
    type Out = D::T;

    #[inline]
    fn apply(input: Self::In) -> Self {
        Self {
            result: D::def(&input),
            input,
        }
    }

    #[inline]
    fn result(self) -> Result<Self::Out> {
        match self.result {
            Err(e) => Err(e),
            Ok(None) => Ok(self.input),
            Ok(Some(r)) => Ok(r),
        }
    }
}

#[cfg(test)]
mod test_try_from {
    use crate::core::Callable;
    use std::convert::TryFrom;

    use super::WithTryFrom;
    use pretty_assertions::assert_eq;

    pub struct A(i8);
    #[derive(Debug, PartialEq)]
    pub struct B(String);

    pub struct C(i8);
    #[derive(Debug, PartialEq)]
    pub struct D(String);

    impl From<A> for B {
        fn from(f: A) -> B {
            B(format!("{}", f.0))
        }
    }

    impl TryFrom<C> for D {
        type Error = ();

        fn try_from(f: C) -> Result<D, Self::Error> {
            if f.0 < 10 {
                Ok(D(format!("{}", f.0)))
            } else {
                Err(())
            }
        }
    }

    #[test]
    fn test_try_from_with_from() {
        assert_eq!(
            B("100".into()),
            WithTryFrom::<A, B>::apply(A(100)).result().unwrap()
        )
    }

    #[test]
    fn test_try_from_with_try_from_success() {
        assert_eq!(
            D("9".into()),
            WithTryFrom::<C, D>::apply(C(9)).result().unwrap()
        )
    }

    #[test]
    fn test_try_from_with_try_from_fail() {
        assert_eq!(
            "Fail in conv",
            &format!(
                "{}",
                WithTryFrom::<C, D>::apply(C(10)).result().unwrap_err()
            )[..12]
        )
    }
}

#[cfg(test)]
mod test_try_from_ref {
    use crate::core::Callable;
    use std::convert::TryFrom;

    use super::WithTryFromRef;
    use pretty_assertions::assert_eq;

    pub struct A(i8);
    #[derive(Debug, PartialEq)]
    pub struct B(String);

    pub struct C(i8);
    #[derive(Debug, PartialEq)]
    pub struct D(String);

    impl From<&A> for B {
        fn from(f: &A) -> B {
            B(format!("{}", f.0))
        }
    }

    impl TryFrom<&C> for D {
        type Error = ();

        fn try_from(f: &C) -> Result<D, Self::Error> {
            if f.0 < 10 {
                Ok(D(format!("{}", f.0)))
            } else {
                Err(())
            }
        }
    }

    #[test]
    fn test_try_from_ref_with_from() {
        assert_eq!(
            B("100".into()),
            WithTryFromRef::<A, B>::apply(A(100)).result().unwrap()
        )
    }

    #[test]
    fn test_try_from_ref_with_try_from_success() {
        assert_eq!(
            D("9".into()),
            WithTryFromRef::<C, D>::apply(C(9)).result().unwrap()
        )
    }

    #[test]
    fn test_try_from_ref_with_try_from_fail() {
        assert_eq!(
            "Fail in conv",
            &format!(
                "{}",
                WithTryFromRef::<C, D>::apply(C(10)).result().unwrap_err()
            )[..12]
        )
    }
}

#[cfg(test)]
mod test_option_map {
    use pretty_assertions::assert_eq;

    use crate::core::Callable;
    use crate::result::Result;

    use super::{OptionMap, OptionMapRefDef};

    #[derive(Debug, PartialEq)]
    struct A(i8);

    struct Define {}

    impl OptionMapRefDef for Define {
        type T = A;
        fn def(input: &A) -> Result<Option<A>> {
            let val = input.0;
            if val < 10 {
                Ok(None)
            } else if val < 20 {
                Ok(Some(A(val + 1)))
            } else {
                use std::io::*;
                Err(Error::new(ErrorKind::Other, format!("{}", val)).into())
            }
        }
    }

    #[test]
    fn test_option_map_keep() {
        assert_eq!(A(1), OptionMap::<Define>::apply(A(1)).result().unwrap())
    }

    #[test]
    fn test_option_map_replace() {
        assert_eq!(A(11), OptionMap::<Define>::apply(A(10)).result().unwrap())
    }

    #[test]
    fn test_option_map_fail() {
        assert_eq!(
            "20",
            format!(
                "{}",
                OptionMap::<Define>::apply(A(20)).result().unwrap_err()
            )
        )
    }
}
