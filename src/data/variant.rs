use std::collections::HashMap;

use lazy_static::lazy_static;
use urashima_util::{num::Signed, Index};

use super::{symbol, Function, Int, Invoke, Nat, NativeMethod, Record, Symbol};
use crate::{
    capsule::Capsule,
    error::{Error, Fallible},
};

#[derive(Clone)]
pub enum Variant {
    Bool(bool),
    Int(Int),
    Nat(Nat),
    Str(String),
    Record(Record),
    Fn(Index<Function>),
    Ref(Index<Variant>),
}

#[allow(dead_code)]
impl Variant {
    pub fn unit() -> Self {
        Variant::Record(Record::unit())
    }

    pub fn typename(&self, _ctx: &mut Capsule<'_>) -> Symbol {
        match self {
            Variant::Bool(_) => symbol!("bool"),
            Variant::Int(_) => symbol!("int"),
            Variant::Nat(_) => symbol!("nat"),
            Variant::Str(_) => symbol!("str"),
            Variant::Record(_) => Symbol::from("()"),
            Variant::Fn(_) => symbol!("fn"),
            Variant::Ref(_) => Symbol::from("ref[\\?]"),
        }
    }

    pub fn as_record(&self) -> Option<&Record> {
        if let Variant::Record(val) = self {
            Some(val)
        } else {
            None
        }
    }

    pub fn as_function<'a>(&self, ctx: &'a Capsule<'_>) -> Option<&'a Function> {
        if let Variant::Fn(idx) = self {
            ctx.environment.get_function(*idx)
        } else {
            None
        }
    }

    pub fn to_bool(&self) -> Option<bool> {
        if let Variant::Bool(val) = self {
            Some(*val)
        } else {
            None
        }
    }

    pub fn to_int(&self) -> Option<&Int> {
        if let Variant::Int(val) = self {
            Some(val)
        } else {
            None
        }
    }

    pub fn invoke(
        &self,
        ctx: &mut Capsule<'_>,
        method: Symbol,
        arguments: &[Variant],
    ) -> Fallible<Variant> {
        match self {
            Variant::Int(val) => {
                let f = VTABLE_INT.get(&method).ok_or_else(|| Error::name(method))?;
                f.invoke(ctx, val, arguments)
            }
            Variant::Str(val) => {
                let f = VTABLE_STR.get(&method).ok_or_else(|| Error::name(method))?;
                f.invoke(ctx, val, arguments)
            }
            _ => Err(Error::name(method)),
        }
    }
}

type VirtualTable<T> = HashMap<Symbol, Box<dyn Invoke<Receiver = T> + Send + Sync + 'static>>;

lazy_static! {
    static ref VTABLE_INT: VirtualTable<Int> = {
        let mut m = VirtualTable::<Int>::new();
        m.insert(
            "abs".into(),
            Box::new(NativeMethod::from(|_: &mut Capsule<'_>, this: &Int| {
                Ok(this.abs().to_biguint().expect("unreachable"))
            })),
        );
        m.insert(
            "negate".into(),
            Box::new(NativeMethod::from(|_: &mut Capsule<'_>, this: &Int| {
                Ok(-this)
            })),
        );
        m.insert(
            "println".into(),
            Box::new(NativeMethod::from(|ctx: &mut Capsule<'_>, this: &Int| {
                ctx.print(format_args!("{}\n", this))
            })),
        );
        m
    };
    static ref VTABLE_STR: VirtualTable<String> = {
        let mut m = VirtualTable::<String>::new();
        m.insert(
            "println".into(),
            Box::new(NativeMethod::from(
                |ctx: &mut Capsule<'_>, this: &String| ctx.print(format_args!("{}\n", this)),
            )),
        );
        m
    };
}

impl From<()> for Variant {
    fn from(_: ()) -> Self {
        Variant::unit()
    }
}

impl From<bool> for Variant {
    fn from(val: bool) -> Self {
        Variant::Bool(val)
    }
}

impl From<Int> for Variant {
    fn from(val: Int) -> Self {
        Variant::Int(val)
    }
}

impl From<Nat> for Variant {
    fn from(val: Nat) -> Self {
        Variant::Nat(val)
    }
}

impl From<&str> for Variant {
    fn from(val: &str) -> Self {
        Variant::Str(val.into())
    }
}

#[cfg(test)]
mod test {
    use std::mem;

    use super::*;

    #[test]
    fn variant_size() {
        assert!(mem::size_of::<Variant>() <= 64);
    }
}
