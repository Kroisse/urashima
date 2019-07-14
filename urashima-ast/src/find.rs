use crate::{
    expr::ExprArena,
    span::{Position, Span},
};

pub trait Find {

    fn find_span(&self, pos: Position, arena: &ExprArena) -> Option<Span>;
}

impl Find for Span {
    fn find_span(&self, pos: Position, _: &ExprArena) -> Option<Span> {
        log::debug!("find_span(Span)");
        if self.contains(pos) {
            Some(*self)
        } else {
            None
        }
    }
}
