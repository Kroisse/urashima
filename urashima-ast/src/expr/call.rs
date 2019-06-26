use urashima_util::Symbol;

#[cfg(feature = "deserialize")]
use serde_derive_urashima::DeserializeSeed;

use super::ExprIndex;
use crate::print::{self, Print};

#[derive(Clone, Debug)]
#[cfg_attr(feature = "deserialize", derive(DeserializeSeed))]
pub struct CallExpression {
    pub callee: ExprIndex,
    #[cfg_attr(feature = "deserialize", serde(default))]
    pub arguments: Vec<ExprIndex>,

    #[cfg_attr(feature = "deserialize", serde(skip))]
    __opaque: (),
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "deserialize", derive(DeserializeSeed))]
pub struct InvokeExpression {
    pub receiver: ExprIndex,
    pub method: Symbol,
    #[cfg_attr(feature = "deserialize", serde(default))]
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
