use super::{ExprArena, Expression};
use crate::{
    error::Fallible,
    parser::{Pairs, Parse, Rule},
    print::{self, Print},
    span::Spanned,
    statement::{impls, Statement},
};

#[derive(Clone)]
#[cfg_attr(any(feature = "dev", test), derive(Debug))]
pub struct BlockExpression {
    statements: Vec<Statement>,

    __opaque: (),
}

impl BlockExpression {
    pub(crate) fn new(statements: Vec<Statement>) -> Self {
        BlockExpression {
            statements,
            __opaque: (),
        }
    }

    pub fn statements(&self) -> &[Statement] {
        if let Some(Spanned {
            node: impls::Statement::Expr(_),
            ..
        }) = self.statements.last()
        {
            &self.statements[..self.statements.len() - 1]
        } else {
            &self.statements[..]
        }
    }

    pub fn returns(&self) -> Option<&Expression> {
        if let Some(Spanned {
            node: impls::Statement::Expr(expr),
            ..
        }) = self.statements.last()
        {
            Some(expr)
        } else {
            None
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

    fn from_pairs<'i>(
        arena: &mut ExprArena,
        _span: pest::Span<'i>,
        pairs: Pairs<'i>,
    ) -> Fallible<Self> {
        let mut statements = vec![];
        for item in pairs {
            statements.push(Statement::from_pair(&mut *arena, item)?);
        }
        Ok(BlockExpression::new(statements))
    }
}

impl Print for BlockExpression {
    fn fmt(&self, f: &mut print::Formatter<'_>) -> print::Result {
        f.write_str("{")?;
        f.indent(|f| {
            f.next_line()?;
            for stmt in &self.statements {
                Print::fmt(stmt, f)?;
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
            Ok(BlockExpression::new(statements))
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
