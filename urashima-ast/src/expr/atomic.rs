use std::fmt;

use naru_symbol::Symbol;

#[cfg(deserialize)]
use serde_derive_urashima::DeserializeSeed;

use super::{Display, ExprArena, ExprIndex, Expression};
use crate::{
    error::Fallible,
    parser::{Pairs, Parse, Rule},
    statement::Statement,
};

#[derive(Clone, Debug)]
#[cfg_attr(deserialize, derive(DeserializeSeed))]
pub enum AtomicExpression {
    False,
    True,
    Integral(i64),
    Str(String),
    Name(Symbol),
    Record(Vec<(Symbol, ExprIndex)>),
    Block(BlockExpression),
    Fn {
        parameters: Vec<Symbol>,
        body: BlockExpression,
    },
}

#[derive(Clone, Debug)]
#[cfg_attr(deserialize, derive(DeserializeSeed))]
pub struct BlockExpression {
    statements: Vec<Statement>,
    returns: ExprIndex,
}

impl BlockExpression {
    pub fn statements(&self) -> &[Statement] {
        &self.statements
    }

    pub fn returns(&self) -> ExprIndex {
        self.returns
    }
}

impl Parse for BlockExpression {
    const RULE: Rule = Rule::grouping_brace;

    fn from_pairs(arena: &mut ExprArena, pairs: Pairs<'_>) -> Fallible<Self> {
        let mut statements = vec![];
        let mut expr = AtomicExpression::Record(vec![]).into();
        for item in pairs {
            match item.as_rule() {
                Rule::statement => {
                    statements.push(Statement::from_pairs(&mut *arena, item.into_inner())?);
                }
                Rule::expression => {
                    expr = Expression::from_pairs(&mut *arena, item.into_inner())?;
                }
                _ => unreachable!("{:?}", item),
            }
        }
        let returns = arena.insert(expr);
        Ok(BlockExpression {
            statements,
            returns,
        })
    }
}

impl<'a> fmt::Display for Display<'a, &AtomicExpression> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            AtomicExpression::False => fmt::Display::fmt("false", f),
            AtomicExpression::True => fmt::Display::fmt("true", f),
            AtomicExpression::Integral(i) => fmt::Display::fmt(&i, f),
            AtomicExpression::Str(s) => fmt::Display::fmt(&s, f),
            _ => unimplemented!(),
        }
    }
}
