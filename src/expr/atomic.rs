use serde::Deserialize;

use super::Expression;
use crate::{
    capsule::Context,
    data::{record::Key, Symbol},
    environment::{Environment, Value},
    error::{ErrorKind, Fallible},
    eval::Evaluate,
    statement::Statement,
};

#[derive(Clone, Debug, Deserialize)]
pub enum AtomicExpression {
    False,
    True,
    Integral(i64),
    Str(String), // ?
    Binding {
        #[serde(default)]
        depth: usize,
        index: usize,
    },
    Record(Vec<(Key, Expression)>),
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
            Str(val) => Ok(Value::from(&val[..])),
            Binding { depth, index } => ctx.lookup(*depth, *index).map(Clone::clone),
            Record(exprs) => eval_record(ctx, &exprs),
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

fn eval_record(ctx: &mut Context<'_>, exprs: &[(Key, Expression)]) -> Fallible<Value> {
    let mut items = Vec::new();
    let mut keys = Vec::new();
    for (key, expr) in exprs {
        let val = expr.eval(ctx)?;
        if let Err(i) = keys.binary_search(key) {
            keys.insert(i, key.clone());
            items.push((key.clone(), val));
        } else {
            return Err(ErrorKind::Value.into());
        }
    }
    Ok(Value::Record(items.into_iter().collect()))
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
    use crate::runtime::Runtime;

    #[test]
    fn eval_unit_record() -> Fallible<()> {
        let expr: Expression = from_value(json!({
            "Record": [],
        }))?;

        let rt = Runtime::new();
        let mut capsule = rt.root_capsule();
        let value = capsule.eval(&expr)?;
        let rec = value.as_record().unwrap();
        assert!(rec.fields.is_empty());

        Ok(())
    }

    #[test]
    fn eval_block() -> Fallible<()> {
        let expr: Expression = from_value(json!({
            "Block": {
                "statements": [
                    {"Binding": ["foo", {"Integral": 42}]},
                ],
                "returns": {"Binding": {"index": 0}},
            },
        }))?;

        let rt = Runtime::new();
        let mut capsule = rt.root_capsule();
        let value = capsule.eval(&expr)?;
        assert_eq!(value.to_int(), Some(42));

        Ok(())
    }

    #[test]
    fn eval_fn() -> Fallible<()> {
        let rt = Runtime::new();
        let mut capsule = rt.root_capsule();

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
                "callee": {"Binding": {"index": 0}},
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
        let rt = Runtime::new();
        let mut capsule = rt.root_capsule();

        let decl: Statement = from_value(json!({
            "Binding": ["increase", {
                "Fn": {
                    "parameters": ["n"],
                    "body": {
                        "statements": [],
                        "returns": {"+": [
                            {"Binding": {"index": 0}},
                            {"Integral": 1},
                        ]},
                    }
                }
            }]
        }))?;
        capsule.eval(&decl)?;

        let code = json!({
            "FunctionCall": {
                "callee": {"Binding": {"index": 0}},
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
        let rt = Runtime::new();
        let mut capsule = rt.root_capsule();

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
                "callee": {"Binding": {"index": 1}},
                "arguments": [{"Integral": 1}],
            },
        });

        let expr: Expression = from_value(code)?;
        let value = capsule.eval(&expr)?;
        assert_eq!(value.to_int(), Some(43));

        Ok(())
    }
}
