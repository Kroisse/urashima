use std::fmt;

use naru_symbol::Symbol;
use serde_derive_urashima::DeserializeSeed;

use super::{Display, ExprIndex};

#[derive(Clone, Debug, DeserializeSeed)]
pub enum OperatorExpression {
    #[serde(rename = "infix")]
    Infix(Symbol, ExprIndex, ExprIndex),

    #[serde(rename = "new")]
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
