use serde::Deserialize;

use crate::capsule::Context;
use crate::environment::Value;
use crate::error::{ErrorKind, Fallible};
use crate::eval::Evaluate;

use super::{BlockExpression, Expression};

#[derive(Clone, Debug, Deserialize)]
pub enum ControlFlowExpression {
    If {
        cond: Box<Expression>,
        then_blk: BlockExpression,
        else_blk: Option<BlockExpression>,
    },
    Loop(BlockExpression),
}

impl Evaluate for ControlFlowExpression {
    type Value = Value;

    fn eval(&self, ctx: &mut Context<'_>) -> Fallible<Self::Value> {
        use ControlFlowExpression::*;
        match self {
            If {
                cond,
                then_blk,
                else_blk,
            } => eval_if(ctx, cond, then_blk, else_blk.as_ref()),
            Loop(blk) => eval_loop(ctx, blk),
        }
    }
}

fn eval_if(
    ctx: &mut Context<'_>,
    cond: &Expression,
    then_blk: &BlockExpression,
    else_blk: Option<&BlockExpression>,
) -> Fallible<Value> {
    if let Value::Bool(c) = cond.eval(ctx)? {
        if c {
            then_blk.eval(&mut ctx.push())
        } else if let Some(e) = else_blk {
            e.eval(&mut ctx.push())
        } else {
            Ok(Value::Unit)
        }
    } else {
        Err(ErrorKind::Type.into())
    }
}

fn eval_loop(ctx: &mut Context<'_>, blk: &BlockExpression) -> Fallible<Value> {
    loop {
        if let Err(e) = blk.eval(ctx) {
            match e.kind() {
                ErrorKind::Break => break,
                ErrorKind::Continue => continue,
                _ => {
                    return Err(e);
                }
            }
        }
    }
    Ok(Value::Unit)
}

#[cfg(test)]
mod test {
    use failure::Fallible;
    use serde_json::{from_value, json};

    use super::*;
    use crate::capsule::Capsule;

    #[test]
    fn eval_if() -> Fallible<()> {
        let mut code = json!({
            "If": {
                "cond": "True",
                "then_blk": {
                    "statements": [],
                    "returns": {"Integral": 1},
                },
                "else_blk": {
                    "statements": [],
                    "returns": {"Integral": 2},
                }
            },
        });

        let mut capsule = Capsule::interactive();

        let expr: Expression = from_value(code.clone())?;
        let value = capsule.eval(&expr)?;
        assert_eq!(value.to_int(), Some(1));

        code["If"]["cond"] = serde_json::Value::String("False".to_owned());
        let expr: Expression = from_value(code)?;
        let value = capsule.eval(&expr)?;
        assert_eq!(value.to_int(), Some(2));

        Ok(())
    }
}
