use urashima_util::Symbol;

#[cfg(deserialize)]
use serde_derive_urashima::DeserializeSeed;

use crate::{
    error::Fallible,
    expr::{AtomicExpression, ExprArena, Expression},
    parser::{ensure_single, Pairs, Parse, Rule},
    program::{Binding, PackageDep},
};

#[derive(Clone, Debug)]
#[cfg_attr(deserialize, derive(DeserializeSeed))]
pub enum Statement {
    Binding(Symbol, Expression),
    Expr(Expression),
    Return(Expression),
    Break,
    Continue,
    Use(PackageDep),
    Print(Expression), // for debug only
}

impl Parse for Statement {
    const RULE: Rule = Rule::statement;

    fn from_pairs(arena: &mut ExprArena, pairs: Pairs<'_>) -> Fallible<Self> {
        let item = ensure_single(pairs);
        match item.as_rule() {
            Rule::break_statement => Ok(Statement::Break),
            Rule::continue_statement => Ok(Statement::Continue),
            Rule::return_statement => {
                let expr = if let Some(ret) = item.into_inner().next() {
                    Expression::from_pair(&mut *arena, ret)?
                } else {
                    AtomicExpression::Record(vec![]).into()
                };
                Ok(Statement::Return(expr))
            }
            Rule::binding_statement => {
                let binding = Binding::from_pairs(&mut *arena, item.into_inner())?;
                Ok(Statement::Binding(binding.name, binding.value))
            }
            Rule::expression => Ok(Statement::Expr(Expression::from_pairs(
                &mut *arena,
                item.into_inner(),
            )?)),
            _ => unreachable!("{:?}", item),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::expr::{AtomicExpression, CallExpression, ExprArena};

    #[test]
    fn break_simple() {
        let mut arena = ExprArena::new();
        assert_pat!(
            Statement::from_str(&mut arena, "break\n").unwrap(),
            Statement::Break => {}
        );
    }

    #[test]
    fn continue_simple() {
        let mut arena = ExprArena::new();
        assert_pat!(
            Statement::from_str(&mut arena, "continue;").unwrap(),
            Statement::Continue => {}
        );
    }

    #[test]
    fn return_simple() {
        let mut arena = ExprArena::new();
        assert_pat!(
            Statement::from_str(&mut arena, "return\n").unwrap(),
            Statement::Return(Expression::Atomic(AtomicExpression::Record(rec))) => { assert_eq!(rec.len(), 0); }
        );
    }

    #[test]
    fn return_numeric() {
        let mut arena = ExprArena::new();
        assert_pat!(
            Statement::from_str(&mut arena, "return 42\n").unwrap(),
            Statement::Return(Expression::Atomic(AtomicExpression::Integral(42))) => { }
        );
    }

    #[test]
    fn binding_simple_numeric() {
        let mut arena = ExprArena::new();
        assert_pat!(
            Statement::from_str(&mut arena, "foo := 42\n").unwrap(),
            Statement::Binding(name, Expression::Atomic(AtomicExpression::Integral(42))) => {
                assert_eq!(&name, "foo");
            }
        );
    }

    #[test]
    fn binding_simple_fn() {
        let mut arena = ExprArena::new();
        assert_pat!(
            Statement::from_str(&mut arena, r#"hello := fn {
                "Hello, world!" println()
            }
            "#).unwrap(),
            Statement::Binding(name, Expression::Atomic(AtomicExpression::Fn { .. })) => {
                assert_eq!(&name, "hello");
            }
        );
    }

    #[test]
    fn expr_simple_invoke() {
        let mut arena = ExprArena::new();
        assert_pat!(
            Statement::from_str(&mut arena, "42 println()\n").unwrap(),
            Statement::Expr(Expression::Call(CallExpression::MethodInvocation { method, .. })) => {
                assert_eq!(&method, "println");
            }
        );
    }
}
