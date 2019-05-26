use serde::Deserialize;

use crate::capsule::Context;
use crate::error::{Error, Fallible};
use crate::eval::Evaluate;
use crate::expr::Expression;

#[derive(Debug, Deserialize)]
pub enum Statement {
    Binding(String, Expression),
    Return(Expression),
    Continue,
    Break,
    Print(Expression), // for debug only
}

impl Evaluate for Statement {
    type Value = ();

    fn eval(&self, ctx: &mut Context<'_>) -> Fallible<Self::Value> {
        match self {
            Statement::Binding(name, expr) => {
                let val = expr.eval(ctx)?;
                ctx.bind(name.as_str(), val);
                Ok(())
            }
            _ => Err(Error::unimplemented()),
        }
    }
}

#[cfg(test)]
mod test {
    use failure::Fallible;
    use serde_json::{from_value, json};

    use super::*;
    use crate::capsule::Capsule;
    use crate::environment::Value;

    #[test]
    fn eval_bind_literal() -> Fallible<()> {
        let stmt: Statement = from_value(json!({
            "Binding": ["foo", {"Literal": {"int": 42}}],
        }))?;

        let mut capsule = Capsule::interactive();
        capsule.eval(&stmt)?;
        let env = capsule.environments.last().unwrap();
        assert_eq!(env.values[0], Value::Int(42));
        assert_eq!(env.names[0], "foo");

        Ok(())
    }
}
