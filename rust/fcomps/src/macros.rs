#[doc(hidden)]
#[macro_export]
macro_rules! __reverse {
    ($macro_id:tt [$hd:ty, $($tl:ty,)*] $($rv:ty,)*) => {
        $crate::__reverse!($macro_id [$($tl,)*] $($rv,)* $hd,)
    };
    ($macro_id:tt [] $($rv:ty,)*) => { $crate::$macro_id!($($rv,)*) };
}

#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! __Seq {
    ($g:ty, $($fl:ty,)+) => { $crate::core::Composit<__Seq! ($($fl,)+), $g> };
    ($f:ty,) => { $f }
}

#[macro_export(local_inner_macros)]
macro_rules! Seq {
    ($t:ty $(,$tl:ty)+ $(,)?) => {$crate::__reverse!(__Seq [$t $(,$tl)+,])};
}

#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! __SeqB {
    ($g:ty, $($fl:ty,)+) => { $crate::behavior::Composit<__SeqB! ($($fl,)+), $g> };
    ($f:ty,) => { $f }
}

#[macro_export(local_inner_macros)]
macro_rules! SeqB {
    ($t:ty $(,$tl:ty)+ $(,)?) => {$crate::__reverse!(__SeqB [$t $(,$tl)+,])};
}

#[cfg(test)]
#[allow(dead_code)]
mod test_seq {
    use pretty_assertions::assert_eq;

    use crate::core::{Callable, RefCall, RefDef};
    use crate::result::Result;

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

    type C1 = Seq!(RefCall<A>, RefCall<B>);
    type C2 = Seq!(RefCall<A>, RefCall<B>, RefCall<C>);
    type C3 = Seq!(RefCall<A>, RefCall<B>, RefCall<C>, RefCall<D>,);

    struct S {
        a: Seq!(RefCall<A>, RefCall<B>, RefCall<C>, RefCall<D>),
    }

    #[test]
    fn test_seq_macro() {
        let _ = C1::apply([1]);
        let _ = C2::apply([1]);
        let _ = C3::apply([1]);
        let _ = S { a: C3::apply([1]) };

        assert_eq!(
            [1, 1, 1, 1, 1],
            <Seq!(RefCall<A>, RefCall<B>, RefCall<C>, RefCall<D>)>::apply([1])
                .result()
                .unwrap()
        );
    }
}

#[cfg(test)]
#[allow(dead_code)]
mod test_seqb {
    use async_trait::async_trait;
    use pretty_assertions::assert_eq;

    use crate::behavior::{Behave, BehaveDef, Behavior};
    use crate::result::Result;

    type TA = [i8; 1];
    type TB = [i8; 2];
    type TC = [i8; 3];
    type TD = [i8; 4];
    type R = [i8; 5];
    type Ctx = ();

    struct A {}

    #[async_trait(?Send)]
    impl BehaveDef for A {
        type In = TA;
        type Out = TB;
        type Ctx = Ctx;
        async fn def(i: TA, _ctx: &Ctx) -> Result<TB> {
            Ok([i[0], 1])
        }
    }

    struct B {}

    #[async_trait(?Send)]
    impl BehaveDef for B {
        type In = TB;
        type Out = TC;
        type Ctx = Ctx;
        async fn def(i: TB, _ctx: &Ctx) -> Result<TC> {
            Ok([i[0], i[1], 1])
        }
    }

    struct C {}

    #[async_trait(?Send)]
    impl BehaveDef for C {
        type In = TC;
        type Out = TD;
        type Ctx = Ctx;
        async fn def(i: TC, _ctx: &Ctx) -> Result<TD> {
            Ok([i[0], i[1], i[2], 1])
        }
    }

    struct D {}

    #[async_trait(?Send)]
    impl BehaveDef for D {
        type In = TD;
        type Out = R;
        type Ctx = Ctx;
        async fn def(i: TD, _ctx: &Ctx) -> Result<R> {
            Ok([i[0], i[1], i[2], i[3], 1])
        }
    }

    type C1 = SeqB!(Behave<A>, Behave<B>);
    type C2 = SeqB!(Behave<A>, Behave<B>, Behave<C>);
    type C3 = SeqB!(Behave<A>, Behave<B>, Behave<C>, Behave<D>,);

    struct S {
        a: SeqB!(Behave<A>, Behave<B>, Behave<C>, Behave<D>),
    }

    #[tokio::test]
    async fn test_seqb_macro() {
        let ctx = ();
        let val = [1];
        let _ = C1::apply(val, &ctx);
        let _ = C2::apply(val, &ctx);
        let _ = C3::apply(val, &ctx);
        let _ = S {
            a: C3::apply(val, &ctx).await,
        };

        assert_eq!(
            [1, 1, 1, 1, 1],
            <SeqB!(Behave<A>, Behave<B>, Behave<C>, Behave<D>)>::apply([1], &ctx)
                .await
                .result()
                .unwrap()
        );
    }
}
