use naru_symbol::Symbol;
use serde_derive_urashima::DeserializeSeed;

use crate::{
    expr::Expression,
    program::PackageDep,
};

#[derive(Clone, Debug, DeserializeSeed)]
pub enum Statement {
    Binding(Symbol, Expression),
    Expr(Expression),
    Return(Expression),
    Break,
    Continue,
    Use(PackageDep),
    Print(Expression), // for debug only
}
