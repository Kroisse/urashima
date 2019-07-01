mod expr;

use urashima_ast::{
    program::{Binding, PackageDep, PackageProgram, ScriptProgram},
    statement::impls::Statement,
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

impl Evaluate for str {
    type Value = ();

    fn eval(&self, ctx: &mut Capsule<'_>) -> Fallible<Self::Value> {
        let prog: ScriptProgram = ctx.parse_sourcecode(self)?;
        prog.eval(ctx)
    }
}

#[cfg(test)]
mod test_stmt {
    use std::io;

    use urashima_ast::Print;

    #[cfg(feature = "deserialize")]
    use serde_json::json;

    use super::*;
    use crate::runtime::Runtime;

    #[test]
    #[ignore]
    fn closure() {
        let s = r#"
x := 42
f := fn {
    x println() -- 이 x는 위의 x를 가리킨다.
}
x := "foo" -- 위의 x 바인딩을 가린다.
x println() --> foo
f() -- 하지만 이 호출은 여전히 42를 출력한다.
        "#;
        let rt = Runtime::new();
        let mut out = Vec::new();
        {
            let w = Box::new(io::Cursor::new(&mut out));
            let mut capsule = Capsule::new(rt.context(), w);
            let prog: ScriptProgram = capsule.parse_sourcecode(&s).unwrap();
            println!("{}", prog.display(&capsule.expr_arena));
            capsule.eval(&prog).unwrap();
        }
        assert_eq!(std::str::from_utf8(&out).unwrap(), "foo\n42\n");
    }

    #[cfg(feature = "deserialize")]
    #[test]
    fn eval_bind_literal() {
        let rt = Runtime::new();
        let mut capsule = rt.root_capsule();
        let stmt: Statement = capsule
            .parse(json!({
                "Binding": ["foo", {"Integral": 42}],
            }))
            .unwrap();
        capsule.eval(&stmt).unwrap();
        let env = capsule.environment;
        assert_eq!(env.values[0].to_int(), Some(&42.into()));
        assert_eq!(&env.names[0], "foo");
    }
}
