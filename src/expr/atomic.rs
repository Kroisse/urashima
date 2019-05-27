use serde::Deserialize;

use crate::capsule::Context;
use crate::environment::{Environment, Symbol, Value};
use crate::error::{Error, Fallible};
use crate::eval::Evaluate;
use crate::statement::Statement;
use super::Expression;

#[derive(Clone, Debug, Deserialize)]
pub enum AtomicExpression {
    False,
    True,
    Integral(i64),
    Binding {
        depth: usize,
        index: usize,
    },
    Record(),
    Block(BlockExpression),
    Fn {
        parameters: Vec<Symbol>,
        body: BlockExpression,
    },
}

impl Evaluate for AtomicExpression {
    type Value = Value;

    fn eval(&self, ctx: &mut Context<'_>) -> Fallible<Self::Value> {
        use AtomicExpression::*;
        match self {
            False => Ok(Value::Bool(false)),
            True => Ok(Value::Bool(true)),
            Integral(val) => Ok(Value::from(*val)),
            Binding { depth, index } => ctx.lookup(*depth, *index).map(Clone::clone),
            Record(..) => Err(Error::unimplemented()),
            Block(blk) => blk.eval(ctx),
            Fn { parameters, body } => expr_fn(ctx, parameters, body),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct BlockExpression {
    statements: Vec<Statement>,
    returns: Box<Expression>,
}

impl BlockExpression {
    pub(crate) fn eval_in_context(&self, ctx: &mut Context<'_>) -> Fallible<Value> {
        for stmt in &self.statements {
            stmt.eval(ctx)?;
        }
        self.returns.eval(ctx)
    }
}

impl Evaluate for BlockExpression {
    type Value = Value;

    fn eval(&self, ctx: &mut Context<'_>) -> Fallible<Self::Value> {
        let mut g = ctx.push();
        self.eval_in_context(&mut g)
    }
}

fn expr_fn(
    _ctx: &mut Context<'_>,
    parameters: &[Symbol],
    body: &BlockExpression,
) -> Fallible<Value> {
    let closure = Environment::default();
    Ok(Value::Fn {
        parameters: parameters.to_vec(),
        closure,
        body: body.clone(),
    })
}

#[cfg(test)]
mod test {
    use failure::Fallible;
    use serde_json::{from_value, json};

    use super::*;
    use crate::capsule::Capsule;

    #[test]
    fn eval_block() -> Fallible<()> {
        let expr: Expression = from_value(json!({
            "Block": {
                "statements": [
                    {"Binding": ["foo", {"Integral": 42}]},
                ],
                "returns": {"Binding": {"depth": 0, "index": 0}},
            },
        }))?;

        let mut capsule = Capsule::interactive();
        let value = capsule.eval(&expr)?;
        assert_eq!(value.to_int(), Some(42));

        Ok(())
    }

    #[test]
    fn eval_fn() -> Fallible<()> {
        let mut capsule = Capsule::interactive();

        let decl: Statement = from_value(json!({
            "Binding": ["answer_to_the_ultimate_question_of_life_the_universe_and_everything", {
                "Fn": {
                    "parameters": [],
                    "body": {
                        "statements": [],
                        "returns": {"Integral": 42},
                    }
                }
            }]
        }))?;
        capsule.eval(&decl)?;

        let code = json!({
            "FunctionCall": {
                "callee": {"Binding": {"depth": 0, "index": 0}},
                "arguments": [],
            },
        });

        let expr: Expression = from_value(code)?;
        let value = capsule.eval(&expr)?;
        assert_eq!(value.to_int(), Some(42));

        Ok(())
    }

    #[test]
    fn eval_fn_args() -> Fallible<()> {
        let mut capsule = Capsule::interactive();

        let decl: Statement = from_value(json!({
            "Binding": ["increase", {
                "Fn": {
                    "parameters": ["n"],
                    "body": {
                        "statements": [],
                        "returns": {"+": [
                            {"Binding": {"depth": 0, "index": 0}},
                            {"Integral": 1},
                        ]},
                    }
                }
            }]
        }))?;
        capsule.eval(&decl)?;

        let code = json!({
            "FunctionCall": {
                "callee": {"Binding": {"depth": 0, "index": 0}},
                "arguments": [{"Integral": 1}],
            },
        });

        let expr: Expression = from_value(code)?;
        let value = capsule.eval(&expr)?;
        assert_eq!(value.to_int(), Some(2));

        Ok(())
    }

    #[test]
    fn eval_fn_args_with_closed_bindings() -> Fallible<()> {
        let mut capsule = Capsule::interactive();

        let stmts: Vec<Statement> = from_value(json!([
            {
                "Binding": ["ANSWER", {"Integral": 42}]
            },
            {
                "Binding": ["increase", {
                    "Fn": {
                        "parameters": ["n"],
                        "body": {
                            "statements": [],
                            "returns": {"+": [
                                {"Binding": {"depth": 0, "index": 0}},
                                {"Binding": {"depth": 1, "index": 0}},
                            ]},
                        }
                    }
                }]
            },
        ]))?;
        for s in &stmts {
            capsule.eval(s)?;
        }

        let code = json!({
            "FunctionCall": {
                "callee": {"Binding": {"depth": 0, "index": 1}},
                "arguments": [{"Integral": 1}],
            },
        });

        let expr: Expression = from_value(code)?;
        let value = capsule.eval(&expr)?;
        assert_eq!(value.to_int(), Some(43));

        Ok(())
    }
}
