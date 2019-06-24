use std::fmt;

use serde_derive_urashima::DeserializeSeed;

use crate::{
    capsule::Capsule,
    data::{symbol, Symbol, Variant},
    error::{Error, Fallible},
    eval::Evaluate,
};

use super::{Display, ExprIndex};

#[derive(Clone, Debug, DeserializeSeed)]
pub enum OperatorExpression {
    #[serde(rename = "infix")]
    Infix(Symbol, ExprIndex, ExprIndex),

    #[serde(rename = "new")]
    New(ExprIndex),
}

impl Evaluate for OperatorExpression {
    type Value = Variant;

    fn eval(&self, ctx: &mut Capsule) -> Fallible<Self::Value> {
        use OperatorExpression::*;
        match self {
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
        }
    }
}

impl<'a> fmt::Display for Display<'a, &OperatorExpression> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            OperatorExpression::Infix(op, a, b) => {
                write!(f, "{} {} {}", self.wrap(*a), op, self.wrap(*b))
            }
            OperatorExpression::New(expr) => write!(f, "new {}", self.wrap(*expr)),
        }
    }
}

#[cfg(test)]
mod test {
    use failure::Fallible;
    use serde_json::json;

    use crate::{expr::ExprIndex, runtime::Runtime};

    #[test]
    fn eval_operator_add() -> Fallible<()> {
        let rt = Runtime::new();
        let mut capsule = rt.root_capsule();
        let expr: ExprIndex = capsule.parse(json!({
            "infix": [
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
            "infix": [
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
