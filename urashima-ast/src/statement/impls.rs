#[cfg(feature = "deserialize")]
use serde_derive_state::DeserializeState;

use crate::{
    error::Fallible,
    expr::{ExprArena, Expression},
    parser::{ensure_single, Pairs, Parse, Rule},
    print::{self, Print},
    program::{Binding, PackageDep},
    span::Span,
};

#[derive(Clone)]
#[cfg_attr(any(feature = "dev", test), derive(Debug))]
#[cfg_attr(feature = "deserialize", derive(DeserializeState))]
#[cfg_attr(feature = "deserialize", serde(deserialize_state = "ExprArena"))]
pub enum Statement {
    Binding(#[cfg_attr(feature = "deserialize", serde(state))] Binding),
    Expr(#[cfg_attr(feature = "deserialize", serde(state))] Expression),
    Return(Span, #[cfg_attr(feature = "deserialize", serde(state))] Expression),
    Break,
    Continue,
    Use(PackageDep),
}

impl Parse for Statement {
    const RULE: Rule = Rule::statement;

    fn from_pairs<'i>(
        arena: &mut ExprArena,
        span: pest::Span<'i>,
        pairs: Pairs<'i>,
    ) -> Fallible<Self> {
        let item = ensure_single(pairs);
        match item.as_rule() {
            Rule::break_statement => Ok(Statement::Break),
            Rule::continue_statement => Ok(Statement::Continue),
            Rule::return_statement => {
                let mut pairs = item.into_inner();
                let keyword = Span::from(&pairs.next().expect("unreachable").as_span());
                let expr = if let Some(ret) = pairs.next() {
                    Expression::from_pair(&mut *arena, ret)?
                } else {
                    Expression::unit(Span::new(keyword.end(), keyword.end()))
                };
                Ok(Statement::Return(keyword, expr))
            }
            Rule::binding_statement => {
                let binding = Binding::from_pairs(&mut *arena, item.as_span(), item.into_inner())?;
                Ok(Statement::Binding(binding))
            }
            Rule::expression => Ok(Statement::Expr(Expression::from_pairs(
                &mut *arena,
                item.as_span(),
                item.into_inner(),
            )?)),
            _ => unreachable!(),
        }
    }
}

impl Print for Statement {
    fn fmt(&self, f: &mut print::Formatter<'_>) -> print::Result {
        use Statement::*;
        match self {
            Binding(b) => Print::fmt(b, f),
            Expr(expr) => Print::fmt(expr, f),
            Return(_, expr) => write!(f, "return {}", f.display(expr)),
            Break => f.write_str("break"),
            Continue => f.write_str("continue"),
            Use(..) => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        expr::{impls::Expression, ExprArena, InvokeExpression},
        span::Spanned,
    };

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
            Statement::Return(_, Spanned { node: Expression::Record(rec), .. }) => { assert_eq!(rec.len(), 0); }
        );
    }

    #[test]
    fn return_numeric() {
        let mut arena = ExprArena::new();
        assert_pat!(
            Statement::from_str(&mut arena, "return 42\n").unwrap(),
            Statement::Return(_, Spanned { node: Expression::Integral(42), .. }) => { }
        );
    }

    #[test]
    fn binding_simple_numeric() {
        let mut arena = ExprArena::new();
        assert_pat!(
            Statement::from_str(&mut arena, "foo := 42\n").unwrap(),
            Statement::Binding(Binding { name, value: Spanned { node: Expression::Integral(42), .. }, .. }) => {
                assert_eq!(&name.node, "foo");
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
            Statement::Binding(Binding { name, value: Spanned { node: Expression::Fn(_), ..}, .. }) => {
                assert_eq!(&name.node, "hello");
            }
        );
    }

    #[test]
    fn expr_simple_invoke() {
        let mut arena = ExprArena::new();
        assert_pat!(
            Statement::from_str(&mut arena, "42 println()\n").unwrap(),
            Statement::Expr(Spanned { node: Expression::Invoke(InvokeExpression { method, .. }), .. }) => {
                assert_eq!(&method.node, "println");
            }
        );
    }
}
