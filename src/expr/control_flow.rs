use serde_derive_urashima::DeserializeSeed;

use crate::{
    capsule::Capsule,
    data::{Variant, symbol},
    error::{ControlFlow, Error, Fallible},
    eval::Evaluate,
};

use super::{BlockExpression, ExprIndex};

#[derive(Clone, Debug, DeserializeSeed)]
pub enum ControlFlowExpression {
    If {
        cond: ExprIndex,
        then_blk: BlockExpression,
        else_blk: Option<BlockExpression>,
    },
    Loop(BlockExpression),
}

impl Evaluate for ControlFlowExpression {
    type Value = Variant;

    fn eval(&self, ctx: &mut Capsule) -> Fallible<Self::Value> {
        use ControlFlowExpression::*;
        match self {
            If {
                cond,
                then_blk,
                else_blk,
            } => eval_if(ctx, *cond, then_blk, else_blk.as_ref()),
            Loop(blk) => eval_loop(ctx, blk),
        }
    }
}

fn eval_if(
    ctx: &mut Capsule,
    cond: ExprIndex,
    then_blk: &BlockExpression,
    else_blk: Option<&BlockExpression>,
) -> Fallible<Variant> {
    if let Variant::Bool(c) = cond.eval(ctx)? {
        if c {
            then_blk.eval(&mut ctx.push())
        } else if let Some(e) = else_blk {
            e.eval(&mut ctx.push())
        } else {
            Ok(Variant::unit())
        }
    } else {
        Err(Error::invalid_type(symbol!("bool")))
    }
}

fn eval_loop(ctx: &mut Capsule, blk: &BlockExpression) -> Fallible<Variant> {
    loop {
        if let Err(e) = blk.eval(ctx) {
            match e.as_control_flow() {
                Some(ControlFlow::Break) => break,
                Some(ControlFlow::Continue) => continue,
                None => {
                    return Err(e);
                }
            }
        }
    }
    Ok(Variant::unit())
}

#[cfg(test)]
mod test {
    use failure::Fallible;
    use serde_json::json;

    use super::*;
    use crate::runtime::Runtime;

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

        let rt = Runtime::new();
        let mut capsule = rt.root_capsule();

        let expr: ExprIndex = capsule.parse(code.clone())?;
        let value = capsule.eval(&expr)?;
        assert_eq!(value.to_int(), Some(1));

        code["If"]["cond"] = serde_json::Value::String("False".to_owned());
        let expr: ExprIndex = capsule.parse(code)?;
        let value = capsule.eval(&expr)?;
        assert_eq!(value.to_int(), Some(2));

        Ok(())
    }
}
