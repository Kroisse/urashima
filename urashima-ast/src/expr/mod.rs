pub mod arena;
pub mod block;
pub mod call;
pub mod control_flow;
pub mod function;
pub mod impls;

use crate::{
    span::{Span, Spanned},
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
