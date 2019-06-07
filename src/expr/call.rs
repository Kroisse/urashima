use serde::Deserialize;

use crate::capsule::Capsule;
use crate::data::{Symbol, Variant};
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
    type Value = Variant;

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
) -> Fallible<Variant> {
    let callee = callee.eval(ctx)?;
    if let Variant::Fn {
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
) -> Fallible<Variant> {
    let receiver = receiver.eval(ctx)?;
    if method == "println" {
        match receiver {
            Variant::Str(s) => {
                ctx.write(s.as_bytes())?;
                ctx.write(b"\n")?;
                Ok(Variant::unit())
            }
            Variant::Int(i) => {
                ctx.write(i.to_string().as_bytes())?;
                ctx.write(b"\n")?;
                Ok(Variant::unit())
            }
            _ => Err(ErrorKind::Unimplemented.into()),
        }
    } else {
        Err(ErrorKind::Unimplemented.into())
    }
}
