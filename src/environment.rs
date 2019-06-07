use std::rc::Rc;
use std::sync::Arc;

use serde::Deserialize;

use crate::{
    data::{record::Record, Symbol},
    error::{Error, Fallible},
    expr::BlockExpression,
};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Value {
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
    Ref(Rc<Value>),
}

impl Value {
    pub fn unit() -> Self {
        Value::Record(Record::default())
    }

    pub fn into_ref(self) -> Value {
        Value::Ref(Rc::new(self))
    }

    pub fn as_record(&self) -> Option<&Record> {
        if let Value::Record(val) = self {
            Some(val)
        } else {
            None
        }
    }

    pub fn to_bool(&self) -> Option<bool> {
        if let Value::Bool(val) = self {
            Some(*val)
        } else {
            None
        }
    }

    pub fn to_int(&self) -> Option<i64> {
        if let Value::Int(val) = self {
            Some(*val)
        } else {
            None
        }
    }
}

impl From<bool> for Value {
    fn from(val: bool) -> Value {
        Value::Bool(val)
    }
}

impl From<i64> for Value {
    fn from(val: i64) -> Value {
        Value::Int(val)
    }
}

impl From<&str> for Value {
    fn from(val: &str) -> Value {
        Value::Str(val.into())
    }
}

impl From<String> for Value {
    fn from(val: String) -> Value {
        Value::Str(val)
    }
}

/// Execution context
#[derive(Clone, Default)]
#[cfg_attr(test, derive(Debug))]
pub struct Environment {
    pub(crate) values: Vec<Value>,
    pub(crate) names: Vec<Symbol>,
    pub(crate) packages: Vec<Arc<Package>>,
}

impl Environment {
    pub(crate) fn bind(&mut self, name: Symbol, value: Value) {
        self.names.push(name);
        self.values.push(value);
    }

    pub(crate) fn lookup_name(&self, name: &Symbol) -> Fallible<&Value> {
        let i = self
            .names
            .iter()
            .position(|n| n == name)
            .ok_or_else(Error::name)?;
        Ok(&self.values[i])
    }
}

#[cfg(not(test))]
impl ::std::fmt::Debug for Environment {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt("Environment", f)
    }
}

#[cfg_attr(test, derive(Debug))]
pub struct Package {
    pub(crate) environment: Environment,
}
