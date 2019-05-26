use serde::Deserialize;

use crate::capsule::Context;
use crate::environment::Value;
use crate::error::{ErrorKind, Fallible};
use crate::eval::Evaluate;
use crate::statement::Statement;

#[derive(Debug, Deserialize)]
pub enum Expression {
    // Atomic expressions
    Literal(Value),
    Binding {
        depth: usize,
        index: usize,
    },
    Record(),

    Operator(Box<OperatorExpression>),

    If {
        cond: Box<Expression>,
        then_blk: BlockExpression,
        else_blk: Option<BlockExpression>,
    },
    Loop(BlockExpression),
    Block(BlockExpression),
}

impl Evaluate for Expression {
    type Value = Value;

    fn eval(&self, ctx: &mut Context<'_>) -> Fallible<Self::Value> {
        use Expression::*;
        match self {
            Literal(val) => Ok(val.clone()),
            Binding { depth, index } => ctx.lookup(*depth, *index).map(Clone::clone),
            Record(..) => Err(ErrorKind::Unimplemented.into()),
            Operator(op) => op.eval(ctx),
            If {
                cond,
                then_blk,
                else_blk,
            } => eval_if(ctx, cond, then_blk, else_blk.as_ref()),
            Loop(blk) => eval_loop(ctx, blk),
            Block(blk) => blk.eval(ctx),
        }
    }
}

#[derive(Debug, Deserialize)]
pub enum OperatorExpression {
    #[serde(rename = "+")]
    Addition(Expression, Expression),

    #[serde(rename = "-")]
    Subtraction(Expression, Expression),

    #[serde(rename = "new")]
    New(Expression),
}

impl Evaluate for OperatorExpression {
    type Value = Value;

    fn eval(&self, ctx: &mut Context<'_>) -> Fallible<Self::Value> {
        use OperatorExpression::*;
        match self {
            Addition(a, b) => {
                let a = a.eval(ctx)?;
                let b = b.eval(ctx)?;
                match (a, b) {
                    (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                    _ => Err(ErrorKind::Type.into()),
                }
            }
            Subtraction(a, b) => {
                let a = a.eval(ctx)?;
                let b = b.eval(ctx)?;
                match (a, b) {
                    (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
                    _ => Err(ErrorKind::Type.into()),
                }
            }
            New(expr) => {
                let val = expr.eval(ctx)?;
                Ok(val.into_ref())
            }
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
    let mut g = ctx.push();
    loop {
        if let Err(e) = blk.eval(&mut g) {
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

#[derive(Debug, Deserialize)]
pub struct BlockExpression {
    statements: Vec<Statement>,
    returns: Box<Expression>,
}

impl Evaluate for BlockExpression {
    type Value = Value;

    fn eval(&self, ctx: &mut Context<'_>) -> Fallible<Self::Value> {
        let mut g = ctx.push();
        for stmt in &self.statements {
            stmt.eval(&mut g)?;
        }
        self.returns.eval(&mut g)
    }
}

#[cfg(test)]
mod test {
    use failure::Fallible;
    use serde_json::{from_value, json};

    use super::*;
    use crate::capsule::Capsule;

    #[test]
    fn eval_operator_add() -> Fallible<()> {
        let expr: Expression = from_value(json!({
            "Operator": {"+": [
                {"Literal": {"int": 1}},
                {"Literal": {"int": 2}},
            ]},
        }))?;

        let mut capsule = Capsule::interactive();
        let value = capsule.eval(&expr)?;
        assert_eq!(value, Value::Int(3));

        Ok(())
    }

    #[test]
    fn eval_operator_sub() -> Fallible<()> {
        let expr: Expression = from_value(json!({
            "Operator": {"-": [
                {"Literal": {"int": 1}},
                {"Literal": {"int": 2}},
            ]},
        }))?;

        let mut capsule = Capsule::interactive();
        let value = capsule.eval(&expr)?;
        assert_eq!(value, Value::Int(-1));

        Ok(())
    }

    #[test]
    fn eval_block() -> Fallible<()> {
        let expr: Expression = from_value(json!({
            "Block": {
                "statements": [
                    {"Binding": ["foo", {"Literal": {"int": 42}}]},
                ],
                "returns": {"Binding": {"depth": 0, "index": 0}},
            },
        }))?;

        let mut capsule = Capsule::interactive();
        let value = capsule.eval(&expr)?;
        assert_eq!(value, Value::Int(42));

        Ok(())
    }

    #[test]
    fn eval_if() -> Fallible<()> {
        let mut code = json!({
            "If": {
                "cond": {"Literal": {"bool": true}},
                "then_blk": {
                    "statements": [],
                    "returns": {"Literal": {"int": 1}},
                },
                "else_blk": {
                    "statements": [],
                    "returns": {"Literal": {"int": 2}},
                }
            },
        });

        let mut capsule = Capsule::interactive();

        let expr: Expression = from_value(code.clone())?;
        let value = capsule.eval(&expr)?;
        assert_eq!(value, Value::Int(1));

        code["If"]["cond"]["Literal"]["bool"] = serde_json::Value::Bool(false);
        let expr: Expression = from_value(code)?;
        let value = capsule.eval(&expr)?;
        assert_eq!(value, Value::Int(2));

        Ok(())
    }
}
