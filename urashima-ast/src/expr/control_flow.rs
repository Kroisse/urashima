#[cfg(feature = "deserialize")]
use serde_derive_state::DeserializeState;

use super::{BlockExpression, ExprArena, ExprIndex};
use crate::{
    error::Fallible,
    find::Find,
    parser::{Pairs, Parse, Rule},
    span::{Position, Span, Spanned},
};

#[derive(Clone)]
#[cfg_attr(any(feature = "dev", test), derive(Debug))]
#[cfg_attr(feature = "deserialize", derive(DeserializeState))]
#[cfg_attr(feature = "deserialize", serde(deserialize_state = "ExprArena"))]
pub struct IfExpression {
    #[cfg_attr(feature = "deserialize", serde(skip))]
    if_keyword: Span,
    #[cfg_attr(feature = "deserialize", serde(state))]
    pub cond: Spanned<ExprIndex>,
    #[cfg_attr(feature = "deserialize", serde(state))]
    pub then_blk: BlockExpression,
    // #[cfg_attr(feature = "deserialize", serde(state))]
    // pub elseif: Vec<(ExprIndex, BlockExpression)>,
    #[cfg_attr(feature = "deserialize", serde(state))]
    pub else_blk: Option<BlockExpression>,
}

#[derive(Clone)]
#[cfg_attr(any(feature = "dev", test), derive(Debug))]
#[cfg_attr(feature = "deserialize", derive(DeserializeState))]
#[cfg_attr(feature = "deserialize", serde(deserialize_state = "ExprArena"))]
pub struct LoopExpression {
    #[cfg_attr(feature = "deserialize", serde(skip))]
    loop_keyword: Span,
    #[cfg_attr(feature = "deserialize", serde(state))]
    pub blk: BlockExpression,
}

impl Parse for IfExpression {
    const RULE: Rule = Rule::if_expression;

    fn from_pairs<'i>(
        arena: &mut ExprArena,
        _span: pest::Span<'i>,
        mut pairs: Pairs<'i>,
    ) -> Fallible<Self> {
        let if_keyword = Span::from(&pairs.next().expect("unreachable").as_span());
        let cond = Parse::from_pair(arena, pairs.next().expect("unreachable"))?;
        let then_blk = Parse::from_pair(arena, pairs.next().expect("unreachable"))?;
        let else_blk = pairs
            .next()
            .map(|pair| match pair.as_rule() {
                Rule::if_expression => {
                    let span = pair.as_span();
                    IfExpression::from_pairs(arena, pair.as_span(), pair.into_inner()).map(|expr| {
                        BlockExpression::single(&span, Spanned::new(&span, expr.into()))
                    })
                }
                Rule::grouping_brace => {
                    BlockExpression::from_pairs(arena, pair.as_span(), pair.into_inner())
                }
                _ => unreachable!(),
            })
            .transpose()?;
        Ok(IfExpression {
            if_keyword,
            cond,
            then_blk,
            else_blk,
        })
    }
}

impl Parse for LoopExpression {
    const RULE: Rule = Rule::loop_expression;

    fn from_pairs<'i>(
        arena: &mut ExprArena,
        _span: pest::Span<'i>,
        mut pairs: Pairs<'i>,
    ) -> Fallible<Self> {
        let loop_keyword = Span::from(&pairs.next().expect("unreachable").as_span());
        let blk = Parse::from_pair(arena, pairs.next().expect("unreachable"))?;
        Ok(LoopExpression { loop_keyword, blk })
    }
}

impl Find for IfExpression {
    fn find_span(&self, pos: Position, arena: &ExprArena) -> Option<Span> {
        log::debug!("find_span(IfExpression)");
        self.if_keyword
            .find_span(pos, arena)
            .or_else(|| self.cond.find_span(pos, arena))
            .or_else(|| self.then_blk.find_span(pos, arena))
            .or_else(|| self.else_blk.as_ref().and_then(|e| e.find_span(pos, arena)))
    }
}

impl Find for LoopExpression {
    fn find_span(&self, pos: Position, arena: &ExprArena) -> Option<Span> {
        log::debug!("find_span(LoopExpression)");
        self.blk.find_span(pos, arena)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::expr::{impls::Expression::*, Expression};

    #[test]
    fn if_simple_1() {
        let mut arena = ExprArena::new();
        assert_pat!(
            Expression::from_str(&mut arena, r#"if true { 42 }"#).unwrap(),
            Spanned { node: If(IfExpression { else_blk, .. }), .. } => {
                assert!(else_blk.is_none());
            }
        );
    }

    #[test]
    fn if_else_if_else() {
        let mut arena = ExprArena::new();
        assert_pat!(
            Expression::from_str(&mut arena, r#"if a > 0 {
                42
            } else if a < 0 {
                43
            } else {
                44
            }"#).unwrap(),
            Spanned { node: If(..), .. } => {
                // TODO
            }
        );
    }

    #[test]
    fn loop_minimal() {
        let mut arena = ExprArena::new();
        assert_pat!(
            Expression::from_str(&mut arena, r#"loop {}"#).unwrap(),
            Spanned { node: Loop(..), .. } => {
                // TODO
            }
        );
    }

    #[test]
    fn loop_useless() {
        let mut arena = ExprArena::new();
        assert_pat!(
            Expression::from_str(&mut arena, r#"loop { break }"#).unwrap(),
            Spanned { node: Loop(..), .. } => {
                // TODO
            }
        );
    }
}
