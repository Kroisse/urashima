use urashima_util::Symbol;

use super::ExprIndex;
use crate::print::{self, Print};

#[cfg(feature = "deserialize")]
use serde_derive_state::DeserializeState;

#[cfg(feature = "deserialize")]
use super::ExprArena;

#[derive(Clone)]
#[cfg_attr(test, derive(Debug))]
#[cfg_attr(feature = "deserialize", derive(DeserializeState))]
#[cfg_attr(feature = "deserialize", serde(deserialize_state = "ExprArena"))]
pub struct CallExpression {
    #[cfg_attr(feature = "deserialize", serde(state))]
    pub callee: ExprIndex,
    #[cfg_attr(feature = "deserialize", serde(default, state))]
    pub arguments: Vec<ExprIndex>,

    #[cfg_attr(feature = "deserialize", serde(skip))]
    __opaque: (),
}

#[derive(Clone)]
#[cfg_attr(test, derive(Debug))]
#[cfg_attr(feature = "deserialize", derive(DeserializeState))]
#[cfg_attr(feature = "deserialize", serde(deserialize_state = "ExprArena"))]
pub struct InvokeExpression {
    #[cfg_attr(feature = "deserialize", serde(state))]
    pub receiver: ExprIndex,
    pub method: Symbol,
    #[cfg_attr(feature = "deserialize", serde(default, state))]
    pub arguments: Vec<ExprIndex>,

    #[cfg_attr(feature = "deserialize", serde(skip))]
    __opaque: (),
}

impl InvokeExpression {
    pub(super) fn new(receiver: ExprIndex, method: Symbol, arguments: Vec<ExprIndex>) -> Self {
        InvokeExpression {
            receiver,
            method,
            arguments,
            __opaque: (),
        }
    }
}

impl Print for CallExpression {
    fn fmt(&self, f: &mut print::Formatter<'_>) -> print::Result {
        write!(
            f,
            "{}({})",
            f.display(&self.callee),
            f.display_seq(&self.arguments[..], ", "),
        )
    }
}

impl Print for InvokeExpression {
    fn fmt(&self, f: &mut print::Formatter<'_>) -> print::Result {
        write!(
            f,
            "{} {}({})",
            f.display(&self.receiver),
            self.method,
            f.display_seq(&self.arguments[..], ", "),
        )
    }
}
