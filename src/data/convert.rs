use super::{symbol, Int, Nat, Variant};
use crate::{
    capsule::Capsule,
    error::{Error, Fallible},
};
use urashima_util::Index;

pub trait FromNaru<T>: Sized {
    fn from_naru(val: T, ctx: &mut Capsule) -> Fallible<Self>;
}

impl FromNaru<Variant> for bool {
    fn from_naru(val: Variant, _ctx: &mut Capsule) -> Fallible<Self> {
        val.to_bool()
            .ok_or_else(|| Error::invalid_type(symbol!("bool")))
    }
}

impl FromNaru<Variant> for Int {
    fn from_naru(val: Variant, _ctx: &mut Capsule) -> Fallible<Self> {
        val.to_int()
            .cloned()
            .ok_or_else(|| Error::invalid_type(symbol!("int")))
    }
}

impl FromNaru<Variant> for Nat {
    fn from_naru(val: Variant, _ctx: &mut Capsule) -> Fallible<Self> {
        if let Variant::Nat(val) = val {
            Ok(val.clone())
        } else {
            Err(Error::invalid_type(symbol!("nat")))
        }
    }
}

impl FromNaru<Variant> for String {
    fn from_naru(val: Variant, _ctx: &mut Capsule) -> Fallible<Self> {
        if let Variant::Str(val) = val {
            Ok(val.clone())
        } else {
            Err(Error::invalid_type(symbol!("str")))
        }
    }
}

impl FromNaru<&[Variant]> for () {
    fn from_naru(_val: &[Variant], _ctx: &mut Capsule) -> Fallible<Self> {
        Ok(())
    }
}

impl FromNaru<&[Index<Variant>]> for () {
    fn from_naru(_val: &[Index<Variant>], _ctx: &mut Capsule) -> Fallible<Self> {
        Ok(())
    }
}
