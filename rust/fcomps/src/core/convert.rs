use std::any::type_name;
use std::marker::PhantomData;

use crate::core::{Call, Def, FcompError};
use crate::result::{raise, Result};

pub trait Convertible<T> {
    fn convert(self) -> Result<T>;
}

impl<T, U> Convertible<Option<T>> for Option<U>
where
    U: Convertible<T>,
{
    fn convert(self) -> Result<Option<T>> {
        Ok(match self {
            Some(v) => Some(v.convert()?),
            None => None,
        })
    }
}

pub struct DefConvert<From, To>
where
    From: Convertible<To>,
{
    p: PhantomData<fn() -> (From, To)>,
}

impl<From, To> Def for DefConvert<From, To>
where
    From: Convertible<To>,
{
    type In = From;
    type Out = To;

    #[inline]
    fn def(input: From) -> Result<To> {
        match input.convert() {
            Ok(v) => Ok(v),
            Err(_) => raise(FcompError::ConvertType {
                from: String::from(type_name::<From>()),
                to: String::from(type_name::<To>()),
            }),
        }
    }
}

pub type Convert<From, To> = Call<DefConvert<From, To>>;

pub struct IdentityDef<T> {
    p: PhantomData<fn() -> T>,
}

impl<T> Def for IdentityDef<T> {
    type In = T;
    type Out = T;

    #[inline]
    fn def(input: T) -> Result<T> {
        Ok(input)
    }
}

pub type Identity<T> = Call<IdentityDef<T>>;

#[cfg(test)]
mod test_try_from {
    use super::*;
    use crate::core::Callable;
    use crate::simple_error;
    use pretty_assertions::assert_eq;

    pub struct A(i8);
    #[derive(Debug, PartialEq)]
    pub struct B(String);

    pub struct C(i8);
    #[derive(Debug, PartialEq)]
    pub struct D(String);

    impl Convertible<B> for A {
        fn convert(self) -> Result<B> {
            Ok(B(format!("{}", self.0)))
        }
    }

    impl Convertible<D> for C {
        fn convert(self) -> Result<D> {
            if self.0 < 10 {
                Ok(D(format!("{}", self.0)))
            } else {
                Err(simple_error!("()"))
            }
        }
    }

    #[test]
    fn test_try_from_with_from() {
        assert_eq!(
            B("100".into()),
            Convert::<A, B>::apply(A(100)).result().unwrap()
        )
    }

    #[test]
    fn test_try_from_with_try_from_success() {
        assert_eq!(
            D("9".into()),
            Convert::<C, D>::apply(C(9)).result().unwrap()
        )
    }

    #[test]
    fn test_try_from_with_try_from_fail() {
        assert_eq!(
            "Fail in conv",
            &format!("{}", Convert::<C, D>::apply(C(10)).result().unwrap_err())[..12]
        )
    }
}
