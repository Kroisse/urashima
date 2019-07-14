pub mod impls;

use crate::{
    expr::ExprArena,
    find::Find,
    span::{Position, Span, Spanned},
};

pub type Statement = Spanned<impls::Statement>;

impl Find for Statement {
    fn find_span(&self, pos: Position, arena: &ExprArena) -> Option<Span> {
        log::debug!("find_span(Statement)");
        use self::impls::Statement::*;

        let span = self.span.find_span(pos, arena)?;

        match &self.node {
            Binding(b) => b.find_span(pos, arena),
            Expr(expr) => expr.find_span(pos, arena),
            Return(keyword, expr) => keyword.find_span(pos, arena).or_else(|| expr.find_span(pos, arena)),
            Break | Continue => Some(span),
            Use(pkg) => None,
        }
    }
}
