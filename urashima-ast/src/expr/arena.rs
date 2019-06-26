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

    fn from_pairs(arena: &mut ExprArena, pairs: Pairs<'_>) -> Fallible<Self> {
        let expr = Expression::from_pairs(&mut *arena, pairs)?;
        Ok(arena.insert(expr))
    }
}

impl Print for ExprIndex {
    fn fmt(&self, f: &mut print::Formatter<'_>) -> print::Result {
        let expr = f.get(*self)?;
        Print::fmt(expr, f)
    }
}
