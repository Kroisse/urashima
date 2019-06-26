use urashima_util::Symbol;

#[cfg(feature = "deserialize")]
use serde_derive_urashima::DeserializeSeed;

use super::{BlockExpression, ExprArena};
use crate::{
    error::Fallible,
    parser::{Pairs, Parse, Rule},
};

#[derive(Clone, Debug)]
#[cfg_attr(feature = "deserialize", derive(DeserializeSeed))]
pub struct FunctionExpression {
    pub parameters: Vec<Symbol>,
    pub body: BlockExpression,

    #[cfg_attr(feature = "deserialize", serde(skip))]
    __opaque: (),
}

impl Parse for FunctionExpression {
    const RULE: Rule = Rule::fn_expression;

    fn from_pairs(arena: &mut ExprArena, pairs: Pairs<'_>) -> Fallible<Self> {
        let mut parameters = vec![];
        let mut block: Option<BlockExpression> = None;
        for item in pairs {
            match item.as_rule() {
                Rule::fn_param => {
                    parameters.push(item.as_str().into());
                }
                Rule::grouping_brace => {
                    if block.is_none() {
                        block = Some(BlockExpression::from_pairs(&mut *arena, item.into_inner())?);
                    } else {
                        unreachable!("{:?}", item);
                    }
                }
                _ => unreachable!("{:?}", item),
            }
        }
        Ok(FunctionExpression {
            parameters,
            body: block.expect("unreachable"),
            __opaque: (),
        })
    }
}
