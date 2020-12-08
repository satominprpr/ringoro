#[allow(unused_macros)]
macro_rules! reverse {
    ($macro_id:tt [$hd:ty, $($tl:ty,)*] $($rv:ty,)*) => {
        reverse!($macro_id [$($tl,)*] $($rv,)* $hd,)
    };
    ($macro_id:tt [] $($rv:ty,)*) => { $macro_id!($($rv,)*) };
}

#[allow(unused_macros)]
macro_rules! Sequence_ {
    ($g:ty, $($fl:ty,)+) => { $crate::core::Composit<Sequence_!($($fl,)+), $g> };
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

    use crate::core::{Callable, RefCall, RefDef};
    use crate::result::Result;
    use crate::Sequence;

    type TA = [i8; 1];
    type TB = [i8; 2];
    type TC = [i8; 3];
    type TD = [i8; 4];
    type R = [i8; 5];

    struct A {}

    impl RefDef for A {
        type In = TA;
        type Out = TB;
        fn def(i: &TA) -> Result<TB> {
            Ok([i[0], 1])
        }
    }

    struct B {}

    impl RefDef for B {
        type In = TB;
        type Out = TC;
        fn def(i: &TB) -> Result<TC> {
            Ok([i[0], i[1], 1])
        }
    }

    struct C {}

    impl RefDef for C {
        type In = TC;
        type Out = TD;
        fn def(i: &TC) -> Result<TD> {
            Ok([i[0], i[1], i[2], 1])
        }
    }

    struct D {}

    impl RefDef for D {
        type In = TD;
        type Out = R;
        fn def(i: &TD) -> Result<R> {
            Ok([i[0], i[1], i[2], i[3], 1])
        }
    }

    type C1 = Sequence!(RefCall<A>, RefCall<B>);
    type C2 = Sequence!(RefCall<A>, RefCall<B>, RefCall<C>);
    type C3 = Sequence!(RefCall<A>, RefCall<B>, RefCall<C>, RefCall<D>,);

    struct S {
        a: Sequence!(RefCall<A>, RefCall<B>, RefCall<C>, RefCall<D>),
    }

    #[test]
    fn test_sequence_macro() {
        let _ = C1::apply([1]);
        let _ = C2::apply([1]);
        let _ = C3::apply([1]);
        let _ = S { a: C3::apply([1]) };

        assert_eq!(
            [1, 1, 1, 1, 1],
            <Sequence!(RefCall<A>, RefCall<B>, RefCall<C>, RefCall<D>)>::apply([1])
                .result()
                .unwrap()
        );
    }
}
