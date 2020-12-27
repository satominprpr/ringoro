use std::marker::PhantomData;

use ringoro_utils::result::Result;

pub trait Callable {
    type In;
    type Out;

    fn apply(input: Self::In) -> Self;
    fn result(self) -> Result<Self::Out>;
}

pub trait Def {
    type In;
    type Out;

    fn def(input: Self::In) -> Result<Self::Out>;
}

pub trait RefDef {
    type In;
    type Out;

    fn def(input: &Self::In) -> Result<Self::Out>;
}

pub struct Call<D>
where
    D: Def,
{
    result: Result<D::Out>,
}

impl<D> Callable for Call<D>
where
    D: Def,
{
    type In = D::In;
    type Out = D::Out;

    #[inline]
    fn apply(input: Self::In) -> Self {
        Self {
            result: D::def(input),
        }
    }

    #[inline]
    fn result(self) -> Result<Self::Out> {
        self.result
    }
}

pub struct RefCall<D>
where
    D: RefDef,
{
    result: Result<D::Out>,
    _input: D::In,
}

impl<D> Callable for RefCall<D>
where
    D: RefDef,
{
    type In = D::In;
    type Out = D::Out;

    #[inline]
    fn apply(input: Self::In) -> Self {
        Self {
            result: D::def(&input),
            _input: input,
        }
    }

    #[inline]
    fn result(self) -> Result<Self::Out> {
        self.result
    }
}

pub struct PanicDef<In, Out> {
    p: PhantomData<fn() -> (In, Out)>,
}

impl<In, Out> Def for PanicDef<In, Out> {
    type In = In;
    type Out = Out;

    fn def(_: Self::In) -> Result<Self::Out> {
        panic!()
    }
}

pub type Panic<In, Out> = Call<PanicDef<In, Out>>;

#[cfg(test)]
mod test_call {
    use super::*;
    use pretty_assertions::assert_eq;
    use ringoro_utils::simple_error;

    type In = i8;
    type Out = String;

    struct CallDefSuccess {}

    impl Def for CallDefSuccess {
        type In = In;
        type Out = Out;
        fn def(i: In) -> Result<Out> {
            Ok(format!("{}", i))
        }
    }

    struct CallDefFail {}

    impl Def for CallDefFail {
        type In = In;
        type Out = Out;
        fn def(i: In) -> Result<Out> {
            Err(simple_error!("{}", i))
        }
    }

    #[test]
    fn test_call_define_with_success() {
        let i = 1;
        let runner = Call::<CallDefSuccess>::apply(i);
        assert_eq!("1", runner.result().unwrap());
    }

    #[test]
    fn test_call_define_with_error() {
        let i = 1;
        let runner = Call::<CallDefFail>::apply(i);
        assert_eq!("1", format!("{}", runner.result().unwrap_err()));
    }
}

#[cfg(test)]
mod test_ref_call {
    use super::*;
    use pretty_assertions::assert_eq;
    use ringoro_utils::simple_error;

    type In = i8;
    type Out = String;

    struct RefCallDefSuccess {}

    impl RefDef for RefCallDefSuccess {
        type In = In;
        type Out = Out;
        fn def(i: &In) -> Result<Out> {
            Ok(format!("{}", i))
        }
    }

    struct RefCallDefFail {}

    impl RefDef for RefCallDefFail {
        type In = In;
        type Out = Out;
        fn def(i: &In) -> Result<Out> {
            Err(simple_error!("{}", i))
        }
    }

    #[test]
    fn test_ref_call_define_with_success() {
        let i = 1;
        let runner = RefCall::<RefCallDefSuccess>::apply(i);
        assert_eq!("1", runner.result().unwrap());
    }

    #[test]
    fn test_ref_call_define_with_error() {
        let i = 1;
        let runner = RefCall::<RefCallDefFail>::apply(i);
        assert_eq!("1", format!("{}", runner.result().unwrap_err()));
    }
}
