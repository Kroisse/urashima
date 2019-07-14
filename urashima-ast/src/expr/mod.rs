pub mod arena;
pub mod block;
pub mod call;
pub mod control_flow;
pub mod function;
pub mod impls;

use crate::{
    find::Find,
    span::{Position, Span, Spanned},
    statement::impls::Statement,
};

pub use self::{
    arena::{ExprArena, ExprIndex},
    call::{CallExpression, InvokeExpression},
    control_flow::{IfExpression, LoopExpression},
    function::{FunctionExpression, Parameter},
};

pub type BlockExpression = Spanned<block::BlockExpression>;
pub type Expression = Spanned<impls::Expression>;

impl BlockExpression {
    pub(crate) fn single(span: impl Into<Span>, expr: Expression) -> Self {
        Spanned::new(
            span,
            block::BlockExpression::new(vec![Spanned::new(expr.span, Statement::Expr(expr))]),
        )
    }
}

impl Expression {
    pub(crate) fn unit(span: impl Into<Span>) -> Self {
        Spanned::new(span, impls::Expression::unit())
    }
}

impl Find for Expression {
    fn find_span(&self, pos: Position, arena: &ExprArena) -> Option<Span> {
        log::debug!("find_span(Expression)");
        use self::impls::Expression::*;

        let span = self.span.find_span(pos, arena)?;

        match &self.node {
            // Atomic
            False | True | Integral(_) | Str(_) | Name(_) => Some(span),

            Record(_) => None,
            Block(blk) => blk.find_span(pos, arena),
            Fn(expr) => expr.find_span(pos, arena),

            // Operator
            New(expr) => expr.find_span(pos, arena),
            Infix(op, left, right) => op
                .span
                .find_span(pos, arena)
                .or_else(|| left.find_span(pos, arena))
                .or_else(|| right.find_span(pos, arena)),
            Call(expr) => expr.find_span(pos, arena),
            Invoke(expr) => expr.find_span(pos, arena),

            // Control flow
            If(expr) => expr.find_span(pos, arena),
            Loop(expr) => expr.find_span(pos, arena),
        }
    }
}
