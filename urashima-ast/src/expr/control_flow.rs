#[cfg(feature = "deserialize")]
use serde_derive_state::DeserializeState;

use super::{BlockExpression, ExprArena, ExprIndex};
use crate::{
    error::Fallible,
    parser::{Pairs, Parse, Rule},
};

#[derive(Clone)]
#[cfg_attr(test, derive(Debug))]
#[cfg_attr(feature = "deserialize", derive(DeserializeState))]
#[cfg_attr(feature = "deserialize", serde(deserialize_state = "ExprArena"))]
pub struct IfExpression {
    #[cfg_attr(feature = "deserialize", serde(state))]
    pub cond: ExprIndex,
    #[cfg_attr(feature = "deserialize", serde(state))]
    pub then_blk: BlockExpression,
    // #[cfg_attr(feature = "deserialize", serde(state))]
    // pub elseif: Vec<(ExprIndex, BlockExpression)>,
    #[cfg_attr(feature = "deserialize", serde(state))]
    pub else_blk: Option<BlockExpression>,

    #[cfg_attr(feature = "deserialize", serde(skip))]
    __opaque: (),
}

#[derive(Clone)]
#[cfg_attr(test, derive(Debug))]
#[cfg_attr(feature = "deserialize", derive(DeserializeState))]
#[cfg_attr(feature = "deserialize", serde(deserialize_state = "ExprArena"))]
pub struct LoopExpression {
    #[cfg_attr(feature = "deserialize", serde(state))]
    pub blk: BlockExpression,

    #[cfg_attr(feature = "deserialize", serde(skip))]
    __opaque: (),
}

impl Parse for IfExpression {
    const RULE: Rule = Rule::if_expression;

    fn from_pairs(arena: &mut ExprArena, mut pairs: Pairs<'_>) -> Fallible<Self> {
        let cond = Parse::from_pair(arena, pairs.next().expect("unreachable"))?;
        let then_blk = Parse::from_pair(arena, pairs.next().expect("unreachable"))?;
        let else_blk = pairs
            .next()
            .map(|pair| match pair.as_rule() {
                Rule::if_expression => IfExpression::from_pairs(arena, pair.into_inner())
                    .map(|expr| BlockExpression::single(expr.into())),
                Rule::grouping_brace => BlockExpression::from_pairs(arena, pair.into_inner()),
                _ => unreachable!(),
            })
            .transpose()?;
        Ok(IfExpression {
            cond,
            then_blk,
            else_blk,
            __opaque: (),
        })
    }
}

impl Parse for LoopExpression {
    const RULE: Rule = Rule::loop_expression;

    fn from_pairs(arena: &mut ExprArena, mut pairs: Pairs<'_>) -> Fallible<Self> {
        let blk = Parse::from_pair(arena, pairs.next().expect("unreachable"))?;
        Ok(LoopExpression { blk, __opaque: () })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::expr::Expression;

    #[test]
    fn if_simple_1() {
        let mut arena = ExprArena::new();
        assert_pat!(
            Expression::from_str(&mut arena, r#"if true { 42 }"#).unwrap(),
            Expression::If(IfExpression { else_blk, .. }) => {
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
            Expression::If(..) => {
                // TODO
            }
        );
    }

    #[test]
    fn loop_minimal() {
        let mut arena = ExprArena::new();
        assert_pat!(
            Expression::from_str(&mut arena, r#"loop {}"#).unwrap(),
            Expression::Loop(..) => {
                // TODO
            }
        );
    }

    #[test]
    fn loop_useless() {
        let mut arena = ExprArena::new();
        assert_pat!(
            Expression::from_str(&mut arena, r#"loop { break }"#).unwrap(),
            Expression::Loop(..) => {
                // TODO
            }
        );
    }
}
