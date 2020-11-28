#[allow(unused_macros)]
macro_rules! reverse {
    ($macro_id:tt [$hd:ty, $($tl:ty,)*] $($rv:ty,)*) => {
        reverse!($macro_id [$($tl,)*] $($rv,)* $hd,)
    };
    ($macro_id:tt [] $($rv:ty,)*) => { $macro_id!($($rv,)*) };
}

#[allow(unused_macros)]
macro_rules! Sequence_ {
    ($g:ty, $($fl:ty,)+) => { $crate::fcomps::core::Composit<Sequence_!($($fl,)+), $g> };
    ($f:ty,) => { $f }
}

#[macro_export]
macro_rules! Sequence {
    ($t:ty $(,$tl:ty)+ $(,)?) => {reverse!(Sequence_ [$t $(,$tl)+,])};
}

#[cfg(test)]
#[allow(dead_code)]
mod test_sequence {
    use pretty_assertions::assert_eq;

    use crate::fcomps::core::{Callable, Definer, Effect};
    use crate::result::Result;
    use crate::Sequence;

    type TA = [i8; 1];
    type TB = [i8; 2];
    type TC = [i8; 3];
    type TD = [i8; 4];
    type R = [i8; 5];
    struct Ctx();

    struct A {}

    impl Definer for A {
        type In = TA;
        type Out = TB;
        type Ctx = Ctx;
        fn def(i: &TA, _: &Ctx) -> Result<TB> {
            Ok([i[0], 1])
        }
    }

    struct B {}

    impl Definer for B {
        type In = TB;
        type Out = TC;
        type Ctx = Ctx;
        fn def(i: &TB, _: &Ctx) -> Result<TC> {
            Ok([i[0], i[1], 1])
        }
    }

    struct C {}

    impl Definer for C {
        type In = TC;
        type Out = TD;
        type Ctx = Ctx;
        fn def(i: &TC, _: &Ctx) -> Result<TD> {
            Ok([i[0], i[1], i[2], 1])
        }
    }

    struct D {}

    impl Definer for D {
        type In = TD;
        type Out = R;
        type Ctx = Ctx;
        fn def(i: &TD, _: &Ctx) -> Result<R> {
            Ok([i[0], i[1], i[2], i[3], 1])
        }
    }

    type C1 = Sequence!(Effect<A>, Effect<B>);
    type C2 = Sequence!(Effect<A>, Effect<B>, Effect<C>);
    type C3 = Sequence!(Effect<A>, Effect<B>, Effect<C>, Effect<D>,);

    struct S {
        a: Sequence!(Effect<A>, Effect<B>, Effect<C>, Effect<D>),
    }

    #[test]
    fn test_sequence_macro() {
        let _ = C1::apply(&[1], &Ctx {});
        let _ = C2::apply(&[1], &Ctx {});
        let _ = C3::apply(&[1], &Ctx {});
        let _ = S {
            a: C3::apply(&[1], &Ctx {}),
        };

        assert_eq!(
            [1, 1, 1, 1, 1],
            *<Sequence!(Effect<A>, Effect<B>, Effect<C>, Effect<D>)>::apply(&[1], &Ctx {})
                .result()
                .unwrap()
        );
    }
}
