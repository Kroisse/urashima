use serde_derive_urashima::DeserializeSeed;

use super::{BlockExpression, ExprIndex};

#[derive(Clone, Debug, DeserializeSeed)]
pub enum ControlFlowExpression {
    If {
        cond: ExprIndex,
        then_blk: BlockExpression,
        else_blk: Option<BlockExpression>,
    },
    Loop(BlockExpression),
}
