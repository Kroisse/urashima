use std::marker::PhantomData;

use super::{FromNaru, Variant};
use crate::{
    capsule::Capsule,
    error::{Error, Fallible},
};

pub trait Invoke {
    type Receiver;
    fn invoke(
        &self,
        ctx: &mut Capsule,
        receiver: &Self::Receiver,
        args: &[Variant],
    ) -> Fallible<Variant>;
}

pub struct NativeMethod<F, T, A, R>(F, PhantomData<(T, A, R)>);

macro_rules! impl_native_method {
    (@count $h:ty, $($r:tt,)*) => { 1 + impl_native_method!(@count $($r,)*) };
    (@count) => { 0 };
    (@impls $($t:ident),*) => {
        impl<Func, T, $($t,)* R> From<Func> for NativeMethod<Func, T, ($($t,)*), R>
        where
            Func: Fn(&mut Capsule, &T, $($t),*) -> Fallible<R>,
            R: Into<Variant>
        {
            fn from(f: Func) -> Self { NativeMethod(f, PhantomData) }
        }

        #[allow(non_snake_case, unused_mut, unused_variables)]
        impl<Func, T, $($t,)* R> Invoke for NativeMethod<Func, T, ($($t,)*), R>
        where
            Func: Fn(&mut Capsule, &T, $($t),*) -> Fallible<R>,
            $($t: FromNaru<Variant>,)*
            R: Into<Variant>,
        {
            type Receiver = T;
            fn invoke(&self, ctx: &mut Capsule, receiver: &Self::Receiver, args: &[Variant]) -> Fallible<Variant> {
                if args.len() != impl_native_method!(@count $($t,)*) {
                    return Err(Error::value(""));
                }
                let mut it = args.into_iter();
                $(
                    let $t = <$t as FromNaru<Variant>>::from_naru(it.next().unwrap().clone(), ctx)?;
                )*
                (self.0)(ctx, receiver, $($t,)*).map(Into::into)
            }
        }
    };
    ($t:ident $(, $rest:ident)*) => {
        impl_native_method!(@impls $t $(, $rest)*);
        impl_native_method!($($rest),*);
    };
    () => {
        impl_native_method!(@impls );
    };
}

impl_native_method! { A, B, C, D, E, F, G, H }
