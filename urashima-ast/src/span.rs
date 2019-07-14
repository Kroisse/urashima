use core::cmp::Ordering;
use core::ops::Deref;

use crate::{
    error::Fallible,
    expr::ExprArena,
    parser::{Pairs, Parse, Rule},
    print::{self, Print},
};

#[derive(Clone, Copy, Debug)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub pos: Option<usize>,
}

impl Position {
    pub fn new(line: usize, column: usize) -> Self {
        Self {
            line,
            column,
            pos: None,
        }
    }
}

impl PartialEq for Position {
    fn eq(&self, other: &Position) -> bool {
        (self.line, self.column).eq(&(other.line, other.column))
    }
}

impl Eq for Position {}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Position) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Position) -> Ordering {
        (self.line, self.column).cmp(&(other.line, other.column))
    }
}

impl From<&pest::Position<'_>> for Position {
    fn from(pos: &pest::Position<'_>) -> Self {
        let (line, column) = pos.line_col();
        Position {
            line,
            column,
            pos: Some(pos.pos()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Span {
    start: Position,
    end: Position,
}

impl Span {
    pub fn new(start: Position, end: Position) -> Self {
        assert!(start <= end);
        Self { start, end }
    }

    pub fn enclosing(start: Span, end: Span) -> Self {
        Self::new(start.start(), end.end())
    }

    pub fn start(&self) -> Position { self.start }
    pub fn end(&self) -> Position { self.end }

    pub fn cmp_pos(&self, pos: &Position) -> Ordering {
        use Ordering::*;
        match (self.start.cmp(&pos), self.end.cmp(&pos)) {
            (Less, Less) => Less,
            (Less, Equal) | (Equal, Equal) | (Less, Greater) | (Equal, Greater) => Equal,
            (Greater, Greater) => Greater,
            _ => unreachable!(),
        }
    }

    pub fn contains(&self, pos: Position) -> bool {
        self.start <= pos && pos <= self.end
    }
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn span_cmp_pos() {
        let span = Span::new(Position::new(2, 10), Position::new(2, 16));
        assert_eq!(span.cmp_pos(&Position::new(1, 1)), Ordering::Greater);
        assert_eq!(span.cmp_pos(&Position::new(1, 15)), Ordering::Greater);
        assert_eq!(span.cmp_pos(&Position::new(2, 9)), Ordering::Greater);
        assert_eq!(span.cmp_pos(&Position::new(2, 10)), Ordering::Equal);
        assert_eq!(span.cmp_pos(&Position::new(2, 11)), Ordering::Equal);
        assert_eq!(span.cmp_pos(&Position::new(2, 16)), Ordering::Equal);
        assert_eq!(span.cmp_pos(&Position::new(2, 17)), Ordering::Less);
        assert_eq!(span.cmp_pos(&Position::new(3, 1)), Ordering::Less);
    }
}
