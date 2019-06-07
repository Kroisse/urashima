use serde::Deserialize;

use crate::{
    capsule::Capsule,
    data::Symbol,
    error::{Error, Fallible},
    eval::Evaluate,
    expr::Expression,
    program::PackageDep,
};

#[derive(Clone, Debug, Deserialize)]
pub enum Statement {
    Binding(Symbol, Expression),
    Expr(Expression),
    Return(Expression),
    Break,
    Continue,
    Use(PackageDep),
    Print(Expression), // for debug only
}

impl Evaluate for Statement {
    type Value = ();

    fn eval(&self, ctx: &mut Capsule) -> Fallible<Self::Value> {
        match self {
            Statement::Binding(name, expr) => {
                let val = expr.eval(ctx)?;
                ctx.bind(name.clone(), val);
                Ok(())
            }
            Statement::Expr(expr) => {
                expr.eval(ctx)?;
                Ok(())
            }
            Statement::Break => Err(Error::loop_break()),
            Statement::Continue => Err(Error::loop_continue()),
            Statement::Use(dep) => dep.eval(ctx),
            _ => Err(Error::unimplemented()),
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
    fn eval_bind_literal() -> Fallible<()> {
        let stmt: Statement = from_value(json!({
            "Binding": ["foo", {"Integral": 42}],
        }))?;

        let rt = Runtime::new();
        let mut capsule = rt.root_capsule();
        capsule.eval(&stmt)?;
        let env = capsule.environments.last().unwrap();
        assert_eq!(env.values[0].to_int(), Some(42));
        assert_eq!(&env.names[0], "foo");

        Ok(())
    }
}
