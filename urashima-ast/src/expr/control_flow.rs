#[cfg(feature = "deserialize")]
use serde_derive_urashima::DeserializeSeed;

use super::{BlockExpression, ExprIndex};

#[derive(Clone, Debug)]
#[cfg_attr(feature = "deserialize", derive(DeserializeSeed))]
pub struct IfExpression {
    pub cond: ExprIndex,
    pub then_blk: BlockExpression,
    // pub elseif: Vec<(ExprIndex, BlockExpression)>,
    pub else_blk: Option<BlockExpression>,

    #[cfg_attr(feature = "deserialize", serde(skip))]
    __opaque: (),
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "deserialize", derive(DeserializeSeed))]
pub struct LoopExpression {
    pub blk: BlockExpression,

    #[cfg_attr(feature = "deserialize", serde(skip))]
    __opaque: (),
}
