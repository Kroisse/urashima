use serde::Deserialize;

use crate::capsule::Capsule;
use crate::data::Variant;
use crate::error::{ErrorKind, Fallible};
use crate::eval::Evaluate;

use super::Expression;

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
    type Value = Variant;

    fn eval(&self, ctx: &mut Capsule) -> Fallible<Self::Value> {
        use OperatorExpression::*;
        match self {
            Addition(a, b) => {
                let a = a.eval(ctx)?;
                let b = b.eval(ctx)?;
                match (a, b) {
                    (Variant::Int(a), Variant::Int(b)) => Ok(Variant::Int(a + b)),
                    _ => Err(ErrorKind::Type.into()),
                }
            }
            Subtraction(a, b) => {
                let a = a.eval(ctx)?;
                let b = b.eval(ctx)?;
                match (a, b) {
                    (Variant::Int(a), Variant::Int(b)) => Ok(Variant::Int(a - b)),
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

#[cfg(test)]
mod test {
    use failure::Fallible;
    use serde_json::{from_value, json};

    use super::*;
    use crate::runtime::Runtime;

    #[test]
    fn eval_operator_add() -> Fallible<()> {
        let expr: Expression = from_value(json!({
            "+": [
                {"Integral": 1},
                {"Integral": 2},
            ],
        }))?;

        let rt = Runtime::new();
        let mut capsule = rt.root_capsule();
        let value = capsule.eval(&expr)?;
        assert_eq!(value.to_int(), Some(3));

        Ok(())
    }

    #[test]
    fn eval_operator_sub() -> Fallible<()> {
        let expr: Expression = from_value(json!({
            "-": [
                {"Integral": 1},
                {"Integral": 2},
            ],
        }))?;

        let rt = Runtime::new();
        let mut capsule = rt.root_capsule();
        let value = capsule.eval(&expr)?;
        assert_eq!(value.to_int(), Some(-1));

        Ok(())
    }
}
