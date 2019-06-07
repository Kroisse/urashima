use std::rc::Rc;

use serde::Deserialize;

use crate::{environment::Environment, expr::BlockExpression};
use super::{Symbol, Record};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Variant {
    Bool(bool),
    Int(i64),
    // Nat(u32),
    Str(String),
    Record(Record),

    #[serde(skip)]
    Fn {
        parameters: Vec<Symbol>,
        closure: Environment,
        body: BlockExpression,
    },

    #[serde(skip)]
    Ref(Rc<Variant>),
}

impl Variant {
    pub fn unit() -> Self {
        Variant::Record(Record::default())
    }

    pub fn into_ref(self) -> Self {
        Variant::Ref(Rc::new(self))
    }

    pub fn as_record(&self) -> Option<&Record> {
        if let Variant::Record(val) = self {
            Some(val)
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

impl From<String> for Variant {
    fn from(val: String) -> Self {
        Variant::Str(val)
    }
}
