use lazy_static::lazy_static;

use super::{ExprArena, Expression};
use crate::{
    error::Fallible,
    parser::{Pairs, Parse, Rule},
    print::{self, Print},
    statement::Statement,
};

#[derive(Clone)]
#[cfg_attr(test, derive(Debug))]
pub struct BlockExpression {
    statements: Vec<Statement>,

    __opaque: (),
}

lazy_static! {
    static ref UNIT: Expression = Expression::unit();
}

impl BlockExpression {
    pub(crate) fn single(expr: Expression) -> Self {
        BlockExpression {
            statements: vec![Statement::Expr(expr)],
            __opaque: (),
        }
    }

    pub fn statements(&self) -> &[Statement] {
        if let Some(Statement::Expr(_)) = self.statements.last() {
            &self.statements[..self.statements.len() - 1]
        } else {
            &self.statements[..]
        }
    }

    pub fn returns(&self) -> &Expression {
        if let Some(Statement::Expr(expr)) = self.statements.last() {
            expr
        } else {
            &UNIT
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Statement> {
        self.statements.iter()
    }
}

impl<'a> IntoIterator for &'a BlockExpression {
    type Item = &'a Statement;
    type IntoIter = std::slice::Iter<'a, Statement>;

    fn into_iter(self) -> Self::IntoIter {
        self.statements.iter()
    }
}

impl Parse for BlockExpression {
    const RULE: Rule = Rule::grouping_brace;

    fn from_pairs(arena: &mut ExprArena, pairs: Pairs<'_>) -> Fallible<Self> {
        let mut statements = vec![];
        for item in pairs {
            statements.push(Statement::from_pair(&mut *arena, item)?);
        }
        Ok(BlockExpression {
            statements,
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
            let expr = self.returns();
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

#[cfg(feature = "deserialize")]
mod de {
    use serde_state::de::{DeserializeState, Deserializer};

    use super::*;

    impl<'de> DeserializeState<'de, ExprArena> for BlockExpression {
        fn deserialize_state<D>(seed: &mut ExprArena, deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let statements = Vec::deserialize_state(seed, deserializer)?;
            Ok(BlockExpression {
                statements,
                __opaque: (),
            })
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parse;

    #[test]
    fn inverse() {
        let s = r#"{
    x println()
}"#;
        let mut arena = ExprArena::new();
        let prog: BlockExpression = parse(&mut arena, &s).unwrap();
        let printed = prog.display(&arena).to_string();
        assert_eq!(s, printed);
    }
}
