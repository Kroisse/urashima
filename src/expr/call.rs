use serde::Deserialize;

use crate::capsule::Context;
use crate::environment::Value;
use crate::error::{ErrorKind, Fallible};
use crate::eval::Evaluate;

use super::Expression;

#[derive(Clone, Debug, Deserialize)]
pub enum CallExpression {
    FunctionCall {
        callee: Box<Expression>,
        arguments: Vec<Expression>,
    },
    MethodInvocation {
        object: Box<Expression>,
        method: String,
        arguments: Vec<Expression>,
    },
}

impl Evaluate for CallExpression {
    type Value = Value;

    fn eval(&self, ctx: &mut Context<'_>) -> Fallible<Self::Value> {
        use CallExpression::*;
        match self {
            FunctionCall { callee, arguments } => eval_fn_call(ctx, &callee, &arguments),
            MethodInvocation { .. } => Err(ErrorKind::Unimplemented.into()),
        }
    }
}

fn eval_fn_call(
    ctx: &mut Context<'_>,
    callee: &Expression,
    arguments: &[Expression],
) -> Fallible<Value> {
    let callee = callee.eval(ctx)?;
    if let Value::Fn {
        parameters,
        body,
        closure,
    } = callee
    {
        let args: Vec<_> = arguments
            .iter()
            .map(|arg| arg.eval(ctx))
            .collect::<Result<_, _>>()?;
        let mut g = ctx.push();
        g.load(&closure);
        for (name, val) in parameters.into_iter().zip(args) {
            g.bind(name, val);
        }
        Ok(body.eval_in_context(&mut g)?)
    } else {
        Err(ErrorKind::Type.into())
    }
}
