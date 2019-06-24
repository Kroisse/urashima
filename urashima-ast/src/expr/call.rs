use std::fmt;

use naru_symbol::Symbol;
use serde_derive_urashima::DeserializeSeed;

use super::{Display, ExprIndex};

#[derive(Clone, Debug, DeserializeSeed)]
pub enum CallExpression {
    #[serde(alias = "Call")]
    FunctionCall {
        callee: ExprIndex,
        #[serde(default)]
        arguments: Vec<ExprIndex>,
    },
    #[serde(alias = "Invoke")]
    MethodInvocation {
        receiver: ExprIndex,
        method: Symbol,
        #[serde(default)]
        arguments: Vec<ExprIndex>,
    },
}

impl CallExpression {
    pub(crate) fn invoke(receiver: ExprIndex, method: Symbol, arguments: Vec<ExprIndex>) -> Self {
        CallExpression::MethodInvocation {
            receiver,
            method,
            arguments,
        }
    }
}

impl<'a> fmt::Display for Display<'a, &CallExpression> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            CallExpression::FunctionCall { callee, arguments } => {
                write!(f, "{}({})", &self.wrap(*callee), &self.wrap(&arguments[..]))
            }
            CallExpression::MethodInvocation {
                receiver,
                method,
                arguments,
            } => write!(
                f,
                "{} {}({})",
                &self.wrap(*receiver),
                method,
                &self.wrap(&arguments[..])
            ),
        }
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
