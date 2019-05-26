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
    Binding { depth: usize, index: usize },
    Record(),

    Operator(Box<OperatorExpression>),

    ControlFlow(Box<ControlFlowExpression>),
    Block(BlockExpression),
}

impl Evaluate for Expression {
    type Value = Value;

    fn eval(&self, ctx: &mut Context<'_>) -> Fallible<Self::Value> {
        match self {
            Expression::Literal(val) => Ok(val.clone()),
            Expression::Binding { depth, index } => ctx.lookup(*depth, *index).map(Clone::clone),
            Expression::Operator(op) => op.eval(ctx),
            _ => Err(ErrorKind::Unimplemented.into()),
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

#[derive(Debug, Deserialize)]
pub enum ControlFlowExpression {
    If(Expression, BlockExpression, BlockExpression),
    Loop(BlockExpression),
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
}
