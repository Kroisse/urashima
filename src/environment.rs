use std::rc::Rc;

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Value {
    Unit,
    Bool(bool),
    Int(i64),
    // Nat(u32),
    #[serde(skip)]
    Ref(Rc<Value>),
}

impl Value {
    pub fn into_ref(self) -> Value {
        Value::Ref(Rc::new(self))
    }
}

impl From<bool> for Value {
    fn from(val: bool) -> Value { Value::Bool(val) }
}

impl From<i64> for Value {
    fn from(val: i64) -> Value { Value::Int(val) }
}

/// Execution context
#[derive(Default)]
pub struct Environment {
    pub(crate) values: Vec<Value>,
    pub(crate) names: Vec<String>,
}

impl Environment {
    pub(crate) fn bind(&mut self, name: impl Into<String>, value: Value) {
        self.names.push(name.into());
        self.values.push(value);
    }
}
