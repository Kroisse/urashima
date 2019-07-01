use pest::Parser;
use pest_derive::Parser;

use crate::{
    error::{Error, Fallible},
    expr::ExprArena,
};

#[derive(Parser)]
#[grammar = "parser/syntax.pest"]
pub(crate) struct Syntax;

pub(crate) type Pair<'a> = pest::iterators::Pair<'a, Rule>;
pub(crate) type Pairs<'a> = pest::iterators::Pairs<'a, Rule>;

pub trait Parse: Sized {
    const RULE: Rule;

    fn from_pairs<'i>(
        arena: &mut ExprArena,
        span: pest::Span<'i>,
        pairs: Pairs<'i>,
    ) -> Fallible<Self>;

    fn from_pair(arena: &mut ExprArena, pair: Pair<'_>) -> Fallible<Self> {
        let rule = pair.as_rule();
        if rule == Self::RULE {
            Self::from_pairs(arena, pair.as_span(), pair.into_inner())
        } else {
            Err(Error::unexpected(Self::RULE, rule))
        }
    }

    fn from_str(arena: &mut ExprArena, input: &str) -> Fallible<Self> {
        let pairs = Syntax::parse(Self::RULE, input)?;
        if let Some(item) = pairs.peek() {
            match Self::from_pair(arena, item) {
                Ok(v) => {
                    return Ok(v);
                }
                Err(e) => {
                    if !e.is_unexpected() {
                        return Err(e);
                    }
                }
            }
        }
        Self::from_pairs(
            arena,
            pest::Span::new(input, 0, input.len()).unwrap(),
            pairs,
        )
    }
}

pub(crate) fn ensure_single(mut pairs: Pairs<'_>) -> Pair<'_> {
    let inner = if let Some(first) = pairs.next() {
        first
    } else {
        unreachable!();
    };
    if pairs.next().is_some() {
        unreachable!();
    }
    inner
}

pub fn parse<T>(arena: &mut ExprArena, input: &str) -> Fallible<T>
where
    T: Parse,
{
    <T as Parse>::from_str(arena, input)
}
