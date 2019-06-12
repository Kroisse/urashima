use crate::{arena::Index, capsule::Capsule};

use super::{Function, Record};

#[derive(Clone, Debug)]
pub enum Variant {
    Bool(bool),
    Int(i64),
    // Nat(u32),
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

    pub fn as_record(&self) -> Option<&Record> {
        if let Variant::Record(val) = self {
            Some(val)
        } else {
            None
        }
    }

    pub fn as_function<'a>(&self, ctx: &'a Capsule) -> Option<&'a Function> {
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

    pub fn to_int(&self) -> Option<i64> {
        if let Variant::Int(val) = self {
            Some(*val)
        } else {
            None
        }
    }
}

impl From<bool> for Variant {
    fn from(val: bool) -> Self {
        Variant::Bool(val)
    }
}

impl From<i64> for Variant {
    fn from(val: i64) -> Self {
        Variant::Int(val)
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
        dbg!(mem::size_of::<Variant>());
    }
}
