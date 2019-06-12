use serde_derive_urashima::DeserializeSeed;

use crate::capsule::Capsule;
use crate::data::{Symbol, Variant, symbol};
use crate::error::{Error, Fallible};
use crate::eval::Evaluate;

use super::ExprIndex;

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
    callee: &ExprIndex,
    arguments: &[ExprIndex],
) -> Fallible<Variant> {
    let callee = callee.eval(ctx)?;
    let f = callee
        .as_function(&ctx.fn_arena)
        .ok_or_else(|| Error::invalid_type(symbol!("fn")))?
        .clone();
    f.call(ctx, arguments)
}

fn eval_invoke(
    ctx: &mut Capsule,
    receiver: &ExprIndex,
    method: &Symbol,
    _arguments: &[ExprIndex],
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
            _ => Err(Error::unimplemented()),
        }
    } else {
        Err(Error::unimplemented())
    }
}
