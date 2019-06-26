use std::fmt;

use urashima_util::Symbol;

#[cfg(feature = "deserialize")]
use serde_derive_urashima::DeserializeSeed;

use super::{Display, ExprIndex};

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

impl<'a> fmt::Display for Display<'a, &CallExpression> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}({})",
            &self.wrap(self.value.callee),
            &self.wrap(&self.value.arguments[..])
        )
    }
}

impl<'a> fmt::Display for Display<'a, &InvokeExpression> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}({})",
            &self.wrap(self.value.receiver),
            self.value.method,
            &self.wrap(&self.value.arguments[..])
        )
    }
}

impl<'a> fmt::Display for Display<'a, &[ExprIndex]> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for arg in self.value {
            if first {
                first = false;
            } else {
                fmt::Display::fmt(", ", f)?;
            }
            fmt::Display::fmt(&self.wrap(*arg), f)?;
        }
        Ok(())
    }
}
