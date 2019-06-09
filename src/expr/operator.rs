use serde_derive_urashima::DeserializeSeed;

use crate::capsule::Capsule;
use crate::data::Variant;
use crate::error::{Error, Fallible};
use crate::eval::Evaluate;

use super::ExprIndex;

#[derive(Clone, Debug, DeserializeSeed)]
pub enum OperatorExpression {
    #[serde(rename = "+")]
    Addition(ExprIndex, ExprIndex),

    #[serde(rename = "-")]
    Subtraction(ExprIndex, ExprIndex),

    #[serde(rename = "new")]
    New(ExprIndex),
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
                    _ => Err(Error::invalid_type(symbol!("int"))),
                }
            }
            Subtraction(a, b) => {
                let a = a.eval(ctx)?;
                let b = b.eval(ctx)?;
                match (a, b) {
                    (Variant::Int(a), Variant::Int(b)) => Ok(Variant::Int(a - b)),
                    _ => Err(Error::invalid_type(symbol!("int"))),
                }
            }
            New(expr) => {
                let val = expr.eval(ctx)?;
                Ok(Variant::Ref(ctx.arena.insert(val)))
            }
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
            "+": [
                {"Integral": 1},
                {"Integral": 2},
            ],
        }))?;
        let value = capsule.eval(&expr)?;
        assert_eq!(value.to_int(), Some(3));
        Ok(())
    }

    #[test]
    fn eval_operator_sub() -> Fallible<()> {
        let rt = Runtime::new();
        let mut capsule = rt.root_capsule();
        let expr: ExprIndex = capsule.parse(json!({
            "-": [
                {"Integral": 1},
                {"Integral": 2},
            ],
        }))?;
        let value = capsule.eval(&expr)?;
        assert_eq!(value.to_int(), Some(-1));
        Ok(())
    }
}
