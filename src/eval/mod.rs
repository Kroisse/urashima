mod expr;

use urashima_ast::{
    program::{Binding, PackageDep, PackageProgram, ScriptProgram},
    statement::Statement,
};

use crate::{
    capsule::Capsule,
    error::{Error, Fallible},
};

pub(crate) use self::expr::eval_in_context;

pub trait Evaluate {
    type Value;

    fn eval(&self, ctx: &mut Capsule<'_>) -> Fallible<Self::Value>;
}

impl Evaluate for PackageProgram {
    type Value = ();

    fn eval(&self, ctx: &mut Capsule<'_>) -> Fallible<Self::Value> {
        for dep in &self.uses {
            dep.eval(ctx)?;
        }
        for b in &self.bindings {
            let value = b.value.eval(ctx)?;
            ctx.bind(&b.name, value);
        }
        Ok(())
    }
}

impl Evaluate for PackageDep {
    type Value = ();

    fn eval(&self, ctx: &mut Capsule<'_>) -> Fallible<Self::Value> {
        let pkg = ctx.load(self.path.clone())?;
        for name in &self.imports {
            let value = pkg.environment.lookup_name(&name)?;
            ctx.bind(&name, value.clone());
        }
        Ok(())
    }
}

impl Evaluate for Binding {
    type Value = ();

    fn eval(&self, ctx: &mut Capsule<'_>) -> Fallible<Self::Value> {
        let val = self.value.eval(ctx)?;
        ctx.bind(&self.name, val);
        Ok(())
    }
}

impl Evaluate for ScriptProgram {
    type Value = ();

    fn eval(&self, ctx: &mut Capsule<'_>) -> Fallible<Self::Value> {
        for stmt in &self.statements {
            stmt.eval(ctx)?;
        }
        Ok(())
    }
}

impl Evaluate for Statement {
    type Value = ();

    fn eval(&self, ctx: &mut Capsule<'_>) -> Fallible<Self::Value> {
        match self {
            Statement::Binding(b) => b.eval(ctx),
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

#[cfg(all(feature = "deserialize", test))]
mod test_stmt {
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
