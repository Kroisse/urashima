mod atomic;
mod call;
mod control_flow;
mod operator;

use serde::Deserialize;

use crate::capsule::Capsule;
use crate::environment::Value;
use crate::error::Fallible;
use crate::eval::Evaluate;

pub use self::{
    atomic::{AtomicExpression, BlockExpression},
    call::CallExpression,
    control_flow::ControlFlowExpression,
    operator::OperatorExpression,
};

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Expression {
    Atomic(AtomicExpression),
    Operator(Box<OperatorExpression>),
    Call(CallExpression),
    ControlFlow(ControlFlowExpression),
}

impl Evaluate for Expression {
    type Value = Value;

    fn eval(&self, ctx: &mut Capsule) -> Fallible<Self::Value> {
        use Expression::*;
        match self {
            Atomic(expr) => expr.eval(ctx),
            Operator(expr) => expr.eval(ctx),
            Call(expr) => expr.eval(ctx),
            ControlFlow(expr) => expr.eval(ctx),
        }
    }
}
