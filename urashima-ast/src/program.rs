use urashima_util::{PackagePath, Symbol};

#[cfg(feature = "deserialize")]
use serde_derive::Deserialize;
#[cfg(feature = "deserialize")]
use serde_derive_state::DeserializeState;

use crate::{
    error::Fallible,
    expr::{ExprArena, Expression},
    find::Find,
    parser::{Pairs, Parse, Rule},
    print::{self, Print},
    span::{Position, Span, Spanned},
    statement::Statement,
};

#[cfg_attr(any(feature = "dev", test), derive(Debug))]
#[cfg_attr(feature = "deserialize", derive(DeserializeState))]
#[cfg_attr(feature = "deserialize", serde(deserialize_state = "ExprArena"))]
pub struct PackageProgram {
    /// Dependencies
    pub uses: Vec<PackageDep>,

    ///
    /// Assume that binding declarations are already sorted by topological order.
    ///
    /// https://narucode.org/0/#Binding
    #[cfg_attr(feature = "deserialize", serde(state))]
    pub bindings: Vec<Binding>,
}

#[derive(Clone, PartialEq)]
#[cfg_attr(any(feature = "dev", test), derive(Debug))]
#[cfg_attr(feature = "deserialize", derive(Deserialize))]
pub struct PackageDep {
    pub path: PackagePath,
    pub imports: Vec<Symbol>,
}

#[derive(Clone)]
#[cfg_attr(any(feature = "dev", test), derive(Debug))]
#[cfg_attr(feature = "deserialize", derive(DeserializeState))]
#[cfg_attr(feature = "deserialize", serde(deserialize_state = "ExprArena"))]
pub struct Binding {
    pub name: Spanned<Symbol>,
    #[cfg_attr(feature = "deserialize", serde(skip))]
    bind_op: Span,
    #[cfg_attr(feature = "deserialize", serde(state))]
    pub value: Expression,
}

#[cfg_attr(any(feature = "dev", test), derive(Debug))]
#[cfg_attr(feature = "deserialize", derive(DeserializeState))]
#[cfg_attr(feature = "deserialize", serde(deserialize_state = "ExprArena"))]
pub struct ScriptProgram {
    #[cfg_attr(feature = "deserialize", serde(state))]
    pub statements: Vec<Statement>,
}

impl Parse for PackageProgram {
    const RULE: Rule = Rule::package_program;

    fn from_pairs<'i>(
        arena: &mut ExprArena,
        _span: pest::Span<'i>,
        pairs: Pairs<'i>,
    ) -> Fallible<Self> {
        let mut uses = vec![];
        let mut bindings = vec![];
        for item in pairs {
            match item.as_rule() {
                Rule::use_declaration => {
                    uses.push(PackageDep::from_pairs(
                        &mut *arena,
                        item.as_span(),
                        item.into_inner(),
                    )?);
                }
                Rule::binding_statement => {
                    bindings.push(Binding::from_pairs(
                        &mut *arena,
                        item.as_span(),
                        item.into_inner(),
                    )?);
                }
                Rule::EOI => (),
                _ => unreachable!(),
            }
        }

        Ok(PackageProgram { uses, bindings })
    }
}

impl Parse for PackageDep {
    const RULE: Rule = Rule::use_declaration;

    fn from_pairs<'i>(
        _arena: &mut ExprArena,
        _span: pest::Span<'i>,
        p: Pairs<'i>,
    ) -> Fallible<Self> {
        let mut path: Option<PackagePath> = None;
        let mut imports: Vec<Symbol> = vec![];
        for i in p {
            match i.as_rule() {
                Rule::use_path => {
                    path = Some(
                        i.into_inner()
                            .map(|i| match i.as_rule() {
                                Rule::name => i.as_str(),
                                _ => unreachable!(),
                            })
                            .collect(),
                    );
                }
                Rule::use_imports => imports.extend(i.into_inner().map(|i| match i.as_rule() {
                    Rule::name => i.as_str().into(),
                    _ => unreachable!(),
                })),
                _ => unreachable!(),
            }
        }
        Ok(PackageDep {
            path: path.expect("unreachable"),
            imports,
        })
    }
}

impl Parse for Binding {
    const RULE: Rule = Rule::binding_statement;

    fn from_pairs<'i>(
        arena: &mut ExprArena,
        _span: pest::Span<'i>,
        p: Pairs<'i>,
    ) -> Fallible<Self> {
        let mut name: Option<Spanned<Symbol>> = None;
        let mut bind_op = None;
        let mut value: Option<Expression> = None;
        for i in p {
            match i.as_rule() {
                Rule::name => {
                    if name.is_none() {
                        name = Some(Spanned::new(&i.as_span(), i.as_str().into()));
                    } else {
                        unreachable!();
                    }
                }
                Rule::OPERATOR_BIND => {
                    if bind_op.is_none() {
                        bind_op = Some(Span::from(&i.as_span()));
                    } else {
                        unreachable!();
                    }
                }
                Rule::expression => {
                    if value.is_none() {
                        value = Some(Expression::from_pairs(
                            &mut *arena,
                            i.as_span(),
                            i.into_inner(),
                        )?);
                    } else {
                        unreachable!();
                    }
                }
                _ => unreachable!(),
            }
        }
        Ok(Binding {
            name: name.expect("unreachable"),
            bind_op: bind_op.expect("unreachable"),
            value: value.expect("unreachable"),
        })
    }
}

impl Parse for ScriptProgram {
    const RULE: Rule = Rule::script_program;

    fn from_pairs<'i>(
        arena: &mut ExprArena,
        _span: pest::Span<'i>,
        pairs: Pairs<'i>,
    ) -> Fallible<Self> {
        let mut statements = vec![];
        for item in pairs {
            match item.as_rule() {
                Rule::statement => {
                    statements.push(Statement::from_pairs(
                        &mut *arena,
                        item.as_span(),
                        item.into_inner(),
                    )?);
                }
                Rule::EOI => (),
                _ => unreachable!(),
            }
        }

        Ok(ScriptProgram { statements })
    }
}

impl Print for Binding {
    fn fmt(&self, f: &mut print::Formatter<'_>) -> print::Result {
        write!(f, "{} := ", self.name.node)?;
        Print::fmt(&self.value, f)
    }
}

impl Print for ScriptProgram {
    fn fmt(&self, f: &mut print::Formatter<'_>) -> print::Result {
        for stmt in &self.statements {
            Print::fmt(stmt, f)?;
            f.next_line()?;
        }
        Ok(())
    }
}

impl Find for Binding {
    fn find_span(&self, pos: Position, arena: &ExprArena) -> Option<Span> {
        log::debug!("find_span(Binding): {:?}", pos);
        self.bind_op
            .find_span(pos, arena)
            .or_else(|| self.name.span.find_span(pos, arena))
            .or_else(|| self.value.find_span(pos, arena))
    }
}

impl Find for ScriptProgram {
    fn find_span(&self, pos: Position, arena: &ExprArena) -> Option<Span> {
        log::debug!("find_span(ScriptProgram): {:?}", pos);
        let i = self
            .statements
            .binary_search_by(|s| s.span.cmp_pos(&pos))
            .ok()?;
        self.statements[i].find_span(pos, arena)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{expr::impls::Expression, print::Print};

    #[test]
    fn basic() {
        let mut arena = ExprArena::new();
        let parse_result = PackageProgram::from_str(
            &mut arena,
            r#"
use naru core (bool, int, nat)

foo := 1 + 2  -- this is commment :)
bar := 3 println()
baz := fn {
    42
}
"#,
        )
        .unwrap();
        for b in &parse_result.bindings {
            println!("{}", b.display(&arena));
        }
        assert_eq!(
            parse_result.uses,
            vec![PackageDep {
                path: vec!["naru", "core"].into_iter().collect(),
                imports: vec![
                    Symbol::from("bool"),
                    Symbol::from("int"),
                    Symbol::from("nat"),
                ],
            }]
        );
        assert_eq!(
            parse_result
                .bindings
                .iter()
                .map(|b| &*b.name)
                .collect::<Vec<_>>(),
            vec!["foo", "bar", "baz"],
        );
        let expr = &parse_result.bindings[0].value;
        match &expr.node {
            Expression::Infix(..) => {}
            _ => {
                panic!("{:?}", expr);
            }
        }
    }
}
