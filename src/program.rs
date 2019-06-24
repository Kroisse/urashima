use serde::Deserialize;
use serde_derive_urashima::DeserializeSeed;
use smallvec::SmallVec;

use crate::{
    capsule::Capsule,
    data::Symbol,
    error::Fallible,
    eval::Evaluate,
    expr::{ExprArena, ExprIndex},
    parser::{Pairs, Parse, Rule},
    statement::Statement,
};

pub use self::internal::PackagePath;

#[derive(DeserializeSeed)]
pub struct PackageProgram {
    /// Dependencies
    uses: Vec<PackageDep>,

    ///
    /// Assume that binding declarations are already sorted by topological order.
    ///
    /// https://narucode.org/0/#Binding
    bindings: Vec<Binding>,
}

#[derive(Clone, Debug, DeserializeSeed, PartialEq)]
pub struct PackageDep {
    pub(crate) path: PackagePath,
    pub(crate) imports: Vec<Symbol>,
}

#[derive(DeserializeSeed)]
pub(crate) struct Binding {
    pub(crate) name: Symbol,
    pub(crate) value: ExprIndex,
}

impl Evaluate for PackageProgram {
    type Value = ();

    fn eval(&self, ctx: &mut Capsule) -> Fallible<Self::Value> {
        for dep in &self.uses {
            dep.eval(ctx)?;
        }
        for b in &self.bindings {
            let value = b.value.eval(ctx)?;
            ctx.bind(&b.name, value);
        }
        Ok(())
    }
}

impl Evaluate for PackageDep {
    type Value = ();

    fn eval(&self, ctx: &mut Capsule) -> Fallible<Self::Value> {
        let pkg = ctx.load(self.path.clone())?;
        for name in &self.imports {
            let value = pkg.environment.lookup_name(&name)?;
            ctx.bind(&name, value.clone());
        }
        Ok(())
    }
}

#[derive(DeserializeSeed)]
pub struct ScriptProgram {
    pub(crate) statements: Vec<Statement>,
}

impl Evaluate for ScriptProgram {
    type Value = ();

    fn eval(&self, ctx: &mut Capsule) -> Fallible<Self::Value> {
        for stmt in &self.statements {
            stmt.eval(ctx)?;
        }
        Ok(())
    }
}

mod internal {
    use std::fmt::{self, Display};
    use std::iter::FromIterator;
    use std::slice;

    use serde::de::{DeserializeSeed, Deserializer, SeqAccess, Visitor};

    use super::*;
    use crate::expr::Alloc;

    #[derive(Clone, Debug, Eq, Hash, PartialEq)]
    pub struct PackagePath(SmallVec<[Symbol; 4]>);

    impl<'a> FromIterator<&'a str> for PackagePath {
        fn from_iter<T>(iter: T) -> Self
        where
            T: IntoIterator<Item = &'a str>,
        {
            PackagePath(iter.into_iter().map(Symbol::from).collect())
        }
    }

    impl<'a> IntoIterator for &'a PackagePath {
        type Item = &'a Symbol;
        type IntoIter = slice::Iter<'a, Symbol>;

        fn into_iter(self) -> Self::IntoIter {
            self.0.iter()
        }
    }

    impl<'de> Deserialize<'de> for PackagePath {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct V;
            impl<'a> Visitor<'a> for V {
                type Value = SmallVec<[Symbol; 4]>;

                fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.write_str("an array of strings")
                }

                fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: SeqAccess<'a>,
                {
                    let mut res = seq
                        .size_hint()
                        .map(SmallVec::with_capacity)
                        .unwrap_or_default();
                    while let Some(i) = seq.next_element()? {
                        res.push(i);
                    }
                    Ok(res)
                }
            }
            deserializer.deserialize_seq(V).map(PackagePath)
        }
    }

    impl<'a, 'de> DeserializeSeed<'de> for Alloc<'a, PackagePath> {
        type Value = PackagePath;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            Deserialize::deserialize(deserializer)
        }
    }

    impl Display for PackagePath {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let mut iter = self.0.iter();
            if let Some(s) = iter.next() {
                Display::fmt(&s, f)?;
                for s in iter {
                    f.write_str(" ")?;
                    Display::fmt(&s, f)?;
                }
            }
            Ok(())
        }
    }
}

impl Parse for PackageProgram {
    const RULE: Rule = Rule::package_program;

    fn from_pairs(arena: &mut ExprArena, pairs: Pairs<'_>) -> Fallible<Self> {
        let mut uses = vec![];
        let mut bindings = vec![];
        for item in pairs {
            match item.as_rule() {
                Rule::use_declaration => {
                    uses.push(PackageDep::from_pairs(&mut *arena, item.into_inner())?);
                }
                Rule::binding_statement => {
                    bindings.push(Binding::from_pairs(&mut *arena, item.into_inner())?);
                }
                Rule::EOI => (),
                _ => unreachable!("{:?}", item),
            }
        }

        Ok(PackageProgram { uses, bindings })
    }
}

impl Parse for PackageDep {
    const RULE: Rule = Rule::use_declaration;

    fn from_pairs(_arena: &mut ExprArena, p: Pairs<'_>) -> Fallible<Self> {
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

    fn from_pairs(arena: &mut ExprArena, p: Pairs<'_>) -> Fallible<Self> {
        let mut name: Option<Symbol> = None;
        let mut value: Option<ExprIndex> = None;
        for i in p {
            match i.as_rule() {
                Rule::name => {
                    if name.is_none() {
                        name = Some(i.as_str().into());
                    } else {
                        unreachable!();
                    }
                }
                Rule::expression => {
                    if value.is_none() {
                        value = Some(ExprIndex::from_pairs(&mut *arena, i.into_inner())?);
                    } else {
                        unreachable!();
                    }
                }
                _ => unreachable!("{:?}", i),
            }
        }
        Ok(Binding {
            name: name.expect("unreachable"),
            value: value.expect("unreachable"),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::expr::Expression;

    #[test]
    fn basic() {
        let mut arena = ExprArena::new();
        let parse_result = PackageProgram::from_str(
            &mut arena,
            r#"
use naru core (bool, int, nat)

foo := 1 + 2  -- this is commment :)
bar := 3 println()
"#,
        )
        .unwrap();
        for b in &parse_result.bindings {
            println!("{} := {}", b.name, b.value.display(&arena));
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
            vec!["foo", "bar"],
        );
        let expr = &arena[parse_result.bindings[0].value];
        match expr {
            Expression::Operator(_) => {}
            _ => {
                panic!("{:?}", expr);
            }
        }
    }
}
