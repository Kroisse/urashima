use urashima_util::{Arena, Index};

use super::Expression;
use crate::{
    error::Fallible,
    parser::{Pairs, Parse, Rule},
    print::{self, Print},
};

pub type ExprArena = Arena<Expression>;
pub type ExprIndex = Index<Expression>;

impl Parse for ExprIndex {
    const RULE: Rule = Rule::expression;

    fn from_pairs<'i>(
        arena: &mut ExprArena,
        span: pest::Span<'i>,
        pairs: Pairs<'i>,
    ) -> Fallible<Self> {
        let expr = Expression::from_pairs(&mut *arena, span, pairs)?;
        Ok(arena.insert(expr))
    }
}

impl Print for ExprIndex {
    fn fmt(&self, f: &mut print::Formatter<'_>) -> print::Result {
        let expr = f.get(*self)?;
        Print::fmt(expr, f)
    }
}
