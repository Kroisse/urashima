use serde_derive_urashima::DeserializeSeed;

use crate::capsule::Capsule;
use crate::data::{Symbol, Variant};
use crate::error::{ErrorKind, Fallible};
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
    if let Some(f) = callee.as_function() {
        f.call(ctx, arguments)
    } else {
        Err(ErrorKind::Type.into())
    }
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
            _ => Err(ErrorKind::Unimplemented.into()),
        }
    } else {
        Err(ErrorKind::Unimplemented.into())
    }
}
