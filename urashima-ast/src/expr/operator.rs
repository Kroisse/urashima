use std::fmt;

use naru_symbol::Symbol;

#[cfg(deserialize)]
use serde_derive_urashima::DeserializeSeed;

use super::{Display, ExprIndex};

#[derive(Clone, Debug)]
#[cfg_attr(deserialize, derive(DeserializeSeed))]
pub enum OperatorExpression {
    #[cfg_attr(deserialize, serde(rename = "infix"))]
    Infix(Symbol, ExprIndex, ExprIndex),

    #[cfg_attr(deserialize, serde(rename = "new"))]
    New(ExprIndex),
}

impl<'a> fmt::Display for Display<'a, &OperatorExpression> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            OperatorExpression::Infix(op, a, b) => {
                write!(f, "{} {} {}", self.wrap(*a), op, self.wrap(*b))
            }
            OperatorExpression::New(expr) => write!(f, "new {}", self.wrap(*expr)),
        }
    }
}
