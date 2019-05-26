use serde::Deserialize;

use crate::capsule::Context;
use crate::environment::{Environment, Symbol, Value};
use crate::error::{ErrorKind, Fallible};
use crate::eval::Evaluate;
use crate::statement::Statement;

#[derive(Clone, Debug, Deserialize)]
pub enum Expression {
    // Atomic expressions
    Literal(Value),
    Binding {
        depth: usize,
        index: usize,
    },
    Record(),
    Fn {
        parameters: Vec<Symbol>,
        body: BlockExpression,
    },

    Operator(Box<OperatorExpression>),
    FunctionCall {
        callee: Box<Expression>,
        arguments: Vec<Expression>,
    },
    MethodInvocation {
        object: Box<Expression>,
        method: String,
        arguments: Vec<Expression>,
    },

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
            Fn { parameters, body } => expr_fn(ctx, parameters, body),

            Operator(op) => op.eval(ctx),
            FunctionCall { callee, arguments } => eval_fn_call(ctx, &callee, &arguments),
            MethodInvocation { .. } => Err(ErrorKind::Unimplemented.into()),

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

#[derive(Clone, Debug, Deserialize)]
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

fn eval_fn_call(
    ctx: &mut Context<'_>,
    callee: &Expression,
    arguments: &[Expression],
) -> Fallible<Value> {
    let callee = callee.eval(ctx)?;
    if let Value::Fn {
        parameters,
        body,
        closure,
    } = callee
    {
        let args: Vec<_> = arguments
            .iter()
            .map(|arg| arg.eval(ctx))
            .collect::<Result<_, _>>()?;
        let mut g = ctx.push();
        g.load(&closure);
        for (name, val) in parameters.into_iter().zip(args) {
            g.bind(name, val);
        }
        Ok(body.eval_in_context(&mut g)?)
    } else {
        Err(ErrorKind::Type.into())
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

#[derive(Clone, Debug, Deserialize)]
pub struct BlockExpression {
    statements: Vec<Statement>,
    returns: Box<Expression>,
}

impl BlockExpression {
    fn eval_in_context(&self, ctx: &mut Context<'_>) -> Fallible<Value> {
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
        assert_eq!(value.to_int(), Some(3));

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
        assert_eq!(value.to_int(), Some(-1));

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
        assert_eq!(value.to_int(), Some(42));

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
        assert_eq!(value.to_int(), Some(1));

        code["If"]["cond"]["Literal"]["bool"] = serde_json::Value::Bool(false);
        let expr: Expression = from_value(code)?;
        let value = capsule.eval(&expr)?;
        assert_eq!(value.to_int(), Some(2));

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
                        "returns": {"Literal": {"int": 42}},
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
                        "returns": {"Operator": {"+": [
                            {"Binding": {"depth": 0, "index": 0}},
                            {"Literal": {"int": 1}},
                        ]}},
                    }
                }
            }]
        }))?;
        capsule.eval(&decl)?;

        let code = json!({
            "FunctionCall": {
                "callee": {"Binding": {"depth": 0, "index": 0}},
                "arguments": [{"Literal": {"int": 1}}],
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
                "Binding": ["ANSWER", {
                    "Literal": {"int": 42}
                }]
            },
            {
                "Binding": ["increase", {
                    "Fn": {
                        "parameters": ["n"],
                        "body": {
                            "statements": [],
                            "returns": {"Operator": {"+": [
                                {"Binding": {"depth": 0, "index": 0}},
                                {"Binding": {"depth": 1, "index": 0}},
                            ]}},
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
                "arguments": [{"Literal": {"int": 1}}],
            },
        });

        let expr: Expression = from_value(code)?;
        let value = capsule.eval(&expr)?;
        assert_eq!(value.to_int(), Some(43));

        Ok(())
    }
}
