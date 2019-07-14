use urashima_util::{Arena, Index};

use super::Expression;
use crate::{
    error::Fallible,
    find::Find,
    parser::{Pairs, Parse, Rule},
    print::{self, Print},
    span::{Span, Position}
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

impl Find for ExprIndex {
    fn find_span(&self, pos: Position, arena: &ExprArena) -> Option<Span> {
        log::debug!("find_span(ExprIndex)");
        arena.get(*self).and_then(|e| e.find_span(pos, arena))
    }
}
