use urashima_util::Symbol;

#[cfg(feature = "deserialize")]
use serde_derive::Deserialize;
#[cfg(feature = "deserialize")]
use serde_derive_state::DeserializeState;

use super::{BlockExpression, ExprArena};
use crate::{
    error::Fallible,
    parser::{Pairs, Parse, Rule},
    print::{self, Print},
};

#[derive(Clone, Debug)]
#[cfg_attr(feature = "deserialize", derive(DeserializeState))]
#[cfg_attr(feature = "deserialize", serde(deserialize_state = "ExprArena"))]
pub struct FunctionExpression {
    pub parameters: Vec<Parameter>,
    #[cfg_attr(feature = "deserialize", serde(state))]
    pub body: BlockExpression,

    #[cfg_attr(feature = "deserialize", serde(skip))]
    __opaque: (),
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "deserialize", derive(Deserialize))]
pub struct Parameter(Symbol);

impl Parameter {
    pub fn name(&self) -> Symbol {
        self.0.clone()
    }
}

impl From<&str> for Parameter {
    fn from(s: &str) -> Self {
        Parameter(s.into())
    }
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

impl<'a> Print for FunctionExpression {
    fn fmt(&self, f: &mut print::Formatter<'_>) -> print::Result {
        f.write_str("fn")?;
        if !self.parameters.is_empty() {
            write!(f, "({})", f.display_seq(&self.parameters, ", "))?;
        }
        f.write_str(" ")?;
        Print::fmt(&self.body, f)
    }
}

impl<'a> Print for Parameter {
    fn fmt(&self, f: &mut print::Formatter<'_>) -> print::Result {
        f.write_str(&self.0)
    }
}
