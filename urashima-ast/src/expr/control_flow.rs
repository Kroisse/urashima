#[cfg(deserialize)]
use serde_derive_urashima::DeserializeSeed;

use super::{BlockExpression, ExprIndex};

#[derive(Clone, Debug)]
#[cfg_attr(deserialize, derive(DeserializeSeed))]
pub enum ControlFlowExpression {
    If {
        cond: ExprIndex,
        then_blk: BlockExpression,
        else_blk: Option<BlockExpression>,
    },
    Loop(BlockExpression),
}
