use serde_derive_urashima::DeserializeSeed;

use super::ExprIndex;
use crate::{
    capsule::Capsule,
    data::{record::Key, Function, Symbol, Variant},
    error::{Error, Fallible},
    eval::Evaluate,
    statement::Statement,
};

#[derive(Clone, Debug, DeserializeSeed)]
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
    Record(Vec<(Key, ExprIndex)>),
    Block(BlockExpression),
    Fn {
        parameters: Vec<Symbol>,
        body: BlockExpression,
    },
}

impl<'arena> Evaluate for AtomicExpression {
    type Value = Variant;

    fn eval(&self, ctx: &mut Capsule) -> Fallible<Self::Value> {
        use AtomicExpression::*;
        match self {
            False => Ok(Variant::Bool(false)),
            True => Ok(Variant::Bool(true)),
            Integral(val) => Ok(Variant::from(*val)),
            Str(val) => Ok(Variant::from(&val[..])),
            Binding { depth, index } => ctx.lookup(*depth, *index).map(Clone::clone),
            Record(exprs) => eval_record(ctx, &exprs),
            Block(blk) => blk.eval(ctx),
            Fn { parameters, body } => expr_fn(ctx, parameters, body),
        }
    }
}

#[derive(Clone, Debug, DeserializeSeed)]
pub struct BlockExpression {
    statements: Vec<Statement>,
    returns: ExprIndex,
}

impl BlockExpression {
    pub(crate) fn eval_in_context(&self, ctx: &mut Capsule) -> Fallible<Variant> {
        for stmt in &self.statements {
            stmt.eval(ctx)?;
        }
        self.returns.eval(ctx)
    }
}

impl Evaluate for BlockExpression {
    type Value = Variant;

    fn eval(&self, ctx: &mut Capsule) -> Fallible<Self::Value> {
        let mut g = ctx.push();
        self.eval_in_context(&mut g)
    }
}

fn eval_record(ctx: &mut Capsule, exprs: &[(Key, ExprIndex)]) -> Fallible<Variant> {
    let mut items = Vec::new();
    let mut keys = Vec::new();
    for (key, expr) in exprs {
        if let Err(i) = keys.binary_search(key) {
            keys.insert(i, key.clone());
            let val = expr.eval(ctx)?;
            let idx = ctx.environment.boxed(val);
            items.push((key.clone(), idx));
        } else {
            return Err(Error::value("All labels in the record should be unique"));
        }
    }
    Ok(Variant::Record(items.into_iter().collect()))
}

fn expr_fn(ctx: &mut Capsule, parameters: &[Symbol], body: &BlockExpression) -> Fallible<Variant> {
    let f = Function::new(ctx, parameters.to_vec(), body.clone());
    let idx = ctx.environment.add_function(f);
    Ok(Variant::Fn(idx))
}

#[cfg(test)]
mod test {
    use failure::Fallible;
    use serde_json::json;

    use super::*;
    use crate::runtime::Runtime;

    #[test]
    fn eval_unit_record() -> Fallible<()> {
        let expr = json!({
            "Record": [],
        });

        let rt = Runtime::new();
        let mut capsule = rt.root_capsule();
        let expr: ExprIndex = capsule.parse(expr)?;
        let value = capsule.eval(&expr)?;
        let rec = value.as_record().unwrap();
        assert!(rec.fields.is_empty());

        Ok(())
    }

    #[test]
    fn eval_block() -> Fallible<()> {
        let expr = json!({
            "Block": {
                "statements": [
                    {"Binding": ["foo", {"Integral": 42}]},
                ],
                "returns": {"Binding": {"index": 0}},
            },
        });

        let rt = Runtime::new();
        let mut capsule = rt.root_capsule();
        let expr: ExprIndex = capsule.parse(expr)?;
        let value = capsule.eval(&expr)?;
        assert_eq!(value.to_int(), Some(42));

        Ok(())
    }

    #[test]
    fn eval_fn() -> Fallible<()> {
        let rt = Runtime::new();
        let mut capsule = rt.root_capsule();

        let decl: Statement = capsule.parse(json!({
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
        assert_eq!(capsule.environment.values.len(), 1);

        let code = json!({
            "FunctionCall": {
                "callee": {"Binding": {"index": 0}},
                "arguments": [],
            },
        });

        let expr: ExprIndex = capsule.parse(code)?;
        let value = capsule.eval(&expr)?;
        assert_eq!(value.to_int(), Some(42));

        Ok(())
    }

    #[test]
    fn eval_fn_args() -> Fallible<()> {
        let rt = Runtime::new();
        let mut capsule = rt.root_capsule();

        let decl: Statement = capsule.parse(json!({
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
            "Call": {
                "callee": {"Binding": {"index": 0}},
                "arguments": [{"Integral": 1}],
            },
        });

        let expr: ExprIndex = capsule.parse(code)?;
        let value = capsule.eval(&expr)?;
        assert_eq!(value.to_int(), Some(2));

        Ok(())
    }

    #[test]
    #[ignore]
    fn eval_fn_args_with_closed_bindings() -> Fallible<()> {
        let rt = Runtime::new();
        let mut capsule = rt.root_capsule();

        let stmts = vec![
            json!({
                "Binding": ["ANSWER", {"Integral": 42}]
            }),
            json!({
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
            }),
        ];
        for s in stmts {
            let stmt: Statement = capsule.parse(s)?;
            capsule.eval(&stmt)?;
        }

        let code = json!({
            "Call": {
                "callee": {"Binding": {"index": 1}},
                "arguments": [{"Integral": 1}],
            },
        });

        let expr: ExprIndex = capsule.parse(code)?;
        let value = capsule.eval(&expr)?;
        assert_eq!(value.to_int(), Some(43));

        Ok(())
    }
}
