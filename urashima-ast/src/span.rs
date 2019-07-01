use std::ops::Deref;

use crate::{
    error::Fallible,
    expr::ExprArena,
    parser::{Pairs, Parse, Rule},
    print::{self, Print},
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub pos: usize,
}

impl From<&pest::Position<'_>> for Position {
    fn from(pos: &pest::Position<'_>) -> Self {
        let (line, column) = pos.line_col();
        Position {
            line,
            column,
            pos: pos.pos(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl From<&pest::Span<'_>> for Span {
    fn from(span: &pest::Span<'_>) -> Self {
        Span {
            start: Position::from(&span.start_pos()),
            end: Position::from(&span.end_pos()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Spanned<T> {
    pub span: Span,
    pub node: T,
}

impl<T> Spanned<T> {
    pub fn new(span: impl Into<Span>, node: T) -> Self {
        Spanned {
            span: span.into(),
            node,
        }
    }
}

impl<T> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl<T> Parse for Spanned<T>
where
    T: Parse,
{
    const RULE: Rule = T::RULE;

    fn from_pairs<'i>(
        arena: &mut ExprArena,
        pest_span: pest::Span<'i>,
        pairs: Pairs<'i>,
    ) -> Fallible<Self> {
        let span = Span::from(&pest_span);
        let node = T::from_pairs(arena, pest_span, pairs)?;
        Ok(Spanned { span, node })
    }
}

impl<T> Print for Spanned<T>
where
    T: Print,
{
    fn fmt(&self, f: &mut print::Formatter<'_>) -> print::Result {
        Print::fmt(&self.node, f)
    }
}
