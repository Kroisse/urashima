use serde::Deserialize;

use crate::capsule::Capsule;
use crate::data::Symbol;
use crate::environment::Value;
use crate::error::{ErrorKind, Fallible};
use crate::eval::Evaluate;

use super::Expression;

#[derive(Clone, Debug, Deserialize)]
pub enum CallExpression {
    FunctionCall {
        callee: Box<Expression>,
        #[serde(default)]
        arguments: Vec<Expression>,
    },
    MethodInvocation {
        receiver: Box<Expression>,
        method: Symbol,
        #[serde(default)]
        arguments: Vec<Expression>,
    },
}

impl Evaluate for CallExpression {
    type Value = Value;

    fn eval(&self, ctx: &mut Capsule) -> Fallible<Self::Value> {
        use CallExpression::*;
        match self {
            FunctionCall { callee, arguments } => eval_fn_call(ctx, &callee, &arguments),
            MethodInvocation {
                receiver,
                method,
                arguments,
            } => eval_invoke(ctx, &receiver, &method, &arguments),
        }
    }
}

fn eval_fn_call(
    ctx: &mut Capsule,
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

fn eval_invoke(
    ctx: &mut Capsule,
    receiver: &Expression,
    method: &Symbol,
    _arguments: &[Expression],
) -> Fallible<Value> {
    let receiver = receiver.eval(ctx)?;
    if method == "println" {
        match receiver {
            Value::Str(s) => {
                ctx.write(s.as_bytes())?;
                ctx.write(b"\n")?;
                Ok(Value::unit())
            }
            Value::Int(i) => {
                ctx.write(i.to_string().as_bytes())?;
                ctx.write(b"\n")?;
                Ok(Value::unit())
            }
            _ => Err(ErrorKind::Unimplemented.into()),
        }
    } else {
        Err(ErrorKind::Unimplemented.into())
    }
}
