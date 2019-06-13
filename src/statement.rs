use serde_derive_urashima::DeserializeSeed;

use crate::{
    capsule::Capsule,
    data::Symbol,
    error::{Error, Fallible},
    eval::Evaluate,
    expr::Expression,
    program::PackageDep,
};

#[derive(Clone, Debug, DeserializeSeed)]
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
                ctx.bind(name, val);
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
    use serde_json::json;

    use super::*;
    use crate::runtime::Runtime;

    #[test]
    fn eval_bind_literal() -> Fallible<()> {
        let rt = Runtime::new();
        let mut capsule = rt.root_capsule();
        let stmt: Statement = capsule.parse(json!({
            "Binding": ["foo", {"Integral": 42}],
        }))?;
        capsule.eval(&stmt)?;
        let env = capsule.environment;
        assert_eq!(env.values[0].to_int(), Some(&42.into()));
        assert_eq!(&env.names[0], "foo");
        Ok(())
    }
}
