#[cfg(feature = "deserialize")]
use serde_derive_urashima::DeserializeSeed;

use super::{ExprArena, ExprIndex, Expression};
use crate::{
    error::Fallible,
    parser::{Pairs, Parse, Rule},
    print::{self, Print},
    statement::Statement,
};

#[derive(Clone, Debug)]
#[cfg_attr(feature = "deserialize", derive(DeserializeSeed))]
pub struct BlockExpression {
    statements: Vec<Statement>,
    returns: ExprIndex,

    #[cfg_attr(feature = "deserialize", serde(skip))]
    __opaque: (),
}

impl BlockExpression {
    pub(crate) fn single(expr: ExprIndex) -> Self {
        BlockExpression {
            statements: vec![],
            returns: expr,
            __opaque: (),
        }
    }

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
        let mut expr = Expression::unit();
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
            __opaque: (),
        })
    }
}

impl Print for BlockExpression {
    fn fmt(&self, f: &mut print::Formatter<'_>) -> print::Result {
        f.write_str("{")?;
        f.indent(|f| {
            f.next_line()?;
            for stmt in self.statements() {
                Print::fmt(stmt, f)?;
                f.next_line()?;
            }
            let expr = f.get(self.returns())?;
            if !expr.is_unit() {
                Print::fmt(expr, f)?;
                f.next_line()?;
            }
            Ok(())
        })?;
        f.write_str("}")?;
        Ok(())
    }
}
