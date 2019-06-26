use urashima_ast::expr::{
    BlockExpression, CallExpression, ExprIndex, Expression, FunctionExpression, IfExpression,
    InvokeExpression, LoopExpression,
};

use super::Evaluate;
use crate::{
    capsule::Capsule,
    data::{symbol, Function, Symbol, Variant},
    error::{ControlFlow, Error, Fallible},
};

impl Evaluate for ExprIndex {
    type Value = Variant;

    fn eval(&self, ctx: &mut Capsule) -> Fallible<Self::Value> {
        let expr = ctx
            .expr_arena
            .get(*self)
            .ok_or_else(Error::runtime)?
            .clone();
        expr.eval(ctx)
    }
}

impl Evaluate for Expression {
    type Value = Variant;

    fn eval(&self, ctx: &mut Capsule) -> Fallible<Self::Value> {
        use Expression::*;
        match self {
            False => Ok(Variant::Bool(false)),
            True => Ok(Variant::Bool(true)),
            Integral(val) => Ok(Variant::Int((*val).into())),
            Str(val) => Ok(Variant::from(&val[..])),
            Name(name) => ctx.environment.lookup_name(name).map(Clone::clone),
            Record(exprs) => eval_record(ctx, &exprs),
            Block(blk) => blk.eval(ctx),
            Fn(expr) => expr.eval(ctx),

            Infix(op, a, b) => {
                let a = a.eval(ctx)?;
                let b = b.eval(ctx)?;
                match (op.as_ref(), a, b) {
                    ("+", Variant::Int(a), Variant::Int(b)) => Ok(Variant::Int(a + b)),
                    ("-", Variant::Int(a), Variant::Int(b)) => Ok(Variant::Int(a - b)),
                    ("*", Variant::Int(a), Variant::Int(b)) => Ok(Variant::Int(a * b)),
                    ("/", Variant::Int(a), Variant::Int(b)) => Ok(Variant::Int(a / b)),
                    _ => Err(Error::unimplemented()),
                }
            }
            New(expr) => {
                let val = expr.eval(ctx)?;
                Ok(Variant::Ref(ctx.environment.boxed(val)))
            }
            Call(expr) => expr.eval(ctx),
            Invoke(expr) => expr.eval(ctx),
            If(expr) => expr.eval(ctx),
            Loop(expr) => expr.eval(ctx),
        }
    }
}

impl Evaluate for IfExpression {
    type Value = Variant;

    fn eval(&self, ctx: &mut Capsule) -> Fallible<Self::Value> {
        if let Variant::Bool(c) = self.cond.eval(ctx)? {
            if c {
                self.then_blk.eval(&mut ctx.push())
            } else if let Some(e) = &self.else_blk {
                e.eval(&mut ctx.push())
            } else {
                Ok(Variant::unit())
            }
        } else {
            Err(Error::invalid_type(symbol!("bool")))
        }
    }
}

impl Evaluate for LoopExpression {
    type Value = Variant;

    fn eval(&self, ctx: &mut Capsule) -> Fallible<Self::Value> {
        loop {
            if let Err(e) = self.blk.eval(ctx) {
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
}

impl Evaluate for CallExpression {
    type Value = Variant;

    fn eval(&self, ctx: &mut Capsule) -> Fallible<Self::Value> {
        let callee = self.callee.eval(ctx)?;
        let f = callee
            .as_function(&ctx)
            .ok_or_else(|| Error::invalid_type(symbol!("fn")))?
            .clone();
        f.call(ctx, &self.arguments)
    }
}

impl Evaluate for InvokeExpression {
    type Value = Variant;

    fn eval(&self, ctx: &mut Capsule) -> Fallible<Self::Value> {
        let receiver = self.receiver.eval(ctx)?;
        let arguments = self
            .arguments
            .iter()
            .map(|i| i.eval(ctx))
            .collect::<Fallible<Vec<_>>>()?;
        receiver.invoke(ctx, self.method.clone(), &arguments)
    }
}

impl Evaluate for BlockExpression {
    type Value = Variant;

    fn eval(&self, ctx: &mut Capsule) -> Fallible<Self::Value> {
        let mut g = ctx.push();
        eval_in_context(self, &mut g)
    }
}

pub(crate) fn eval_in_context(expr: &BlockExpression, ctx: &mut Capsule) -> Fallible<Variant> {
    for stmt in expr.statements() {
        stmt.eval(ctx)?;
    }
    expr.returns().eval(ctx)
}

fn eval_record(ctx: &mut Capsule, exprs: &[(Symbol, ExprIndex)]) -> Fallible<Variant> {
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

impl Evaluate for FunctionExpression {
    type Value = Variant;

    fn eval(&self, ctx: &mut Capsule) -> Fallible<Self::Value> {
        let f = Function::new(
            ctx,
            self.parameters.iter().map(|i| i.name()).collect(),
            self.body.clone(),
        );
        let idx = ctx.environment.add_function(f);
        Ok(Variant::Fn(idx))
    }
}

#[cfg(all(feature = "deserialize", test))]
mod test_expr_atomic {
    use failure::Fallible;
    use serde_json::json;
    use urashima_ast::statement::Statement;

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
                "returns": {"Name": "foo"},
            },
        });

        let rt = Runtime::new();
        let mut capsule = rt.root_capsule();
        let expr: ExprIndex = capsule.parse(expr)?;
        let value = capsule.eval(&expr)?;
        assert_eq!(value.to_int(), Some(&42.into()));

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
            "Call": {
                "callee": {"Name": "answer_to_the_ultimate_question_of_life_the_universe_and_everything"},
                "arguments": [],
            },
        });

        let expr: ExprIndex = capsule.parse(code)?;
        let value = capsule.eval(&expr)?;
        assert_eq!(value.to_int(), Some(&42.into()));

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
                        "returns": {"Infix": [
                            "+",
                            {"Name": "n"},
                            {"Integral": 1},
                        ]},
                    }
                }
            }]
        }))?;
        capsule.eval(&decl)?;

        let code = json!({
            "Call": {
                "callee": {"Name": "increase"},
                "arguments": [{"Integral": 1}],
            },
        });

        let expr: ExprIndex = capsule.parse(code)?;
        let value = capsule.eval(&expr)?;
        assert_eq!(value.to_int(), Some(&2.into()));

        Ok(())
    }

    #[test]
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
                            "returns": {"Infix": [
                                "+",
                                {"Name": "n"},
                                {"Name": "ANSWER"},
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
                "callee": {"Name": "increase"},
                "arguments": [{"Integral": 1}],
            },
        });

        let expr: ExprIndex = capsule.parse(code)?;
        let value = capsule.eval(&expr)?;
        assert_eq!(value.to_int(), Some(&43.into()));

        Ok(())
    }
}

#[cfg(all(feature = "deserialize", test))]
mod test_expr_call {
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
        assert_eq!(value.to_int(), Some(&1.into()));

        code["If"]["cond"] = serde_json::Value::String("False".to_owned());
        let expr: ExprIndex = capsule.parse(code)?;
        let value = capsule.eval(&expr)?;
        assert_eq!(value.to_int(), Some(&2.into()));

        Ok(())
    }
}

#[cfg(all(feature = "deserialize", test))]
mod test_expr_op {
    use failure::Fallible;
    use serde_json::json;
    use urashima_ast::expr::ExprIndex;

    use crate::runtime::Runtime;

    #[test]
    fn eval_operator_add() -> Fallible<()> {
        let rt = Runtime::new();
        let mut capsule = rt.root_capsule();
        let expr: ExprIndex = capsule.parse(json!({
            "Infix": [
                "+",
                {"Integral": 1},
                {"Integral": 2},
            ],
        }))?;
        let value = capsule.eval(&expr)?;
        assert_eq!(value.to_int(), Some(&3.into()));
        Ok(())
    }

    #[test]
    fn eval_operator_sub() -> Fallible<()> {
        let rt = Runtime::new();
        let mut capsule = rt.root_capsule();
        let expr: ExprIndex = capsule.parse(json!({
            "Infix": [
                "-",
                {"Integral": 1},
                {"Integral": 2},
            ],
        }))?;
        let value = capsule.eval(&expr)?;
        assert_eq!(value.to_int(), Some(&(-1).into()));
        Ok(())
    }
}
