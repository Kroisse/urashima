use serde::Deserialize;
use smallvec::SmallVec;

use crate::{
    capsule::Context, data::Symbol, error::Fallible, eval::Evaluate, expr::Expression,
    statement::Statement,
};

pub use self::internal::PackagePath;

#[derive(Deserialize)]
pub struct PackageProgram {
    /// Dependencies
    uses: Vec<PackageDep>,

    ///
    /// Assume that binding declarations are already sorted by topological order.
    ///
    /// https://narucode.org/0/#Binding
    bindings: Vec<Binding>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PackageDep {
    pub(crate) path: PackagePath,
    pub(crate) imports: Vec<Symbol>,
}

#[derive(Deserialize)]
struct Binding {
    name: Symbol,
    value: Expression,
}

#[derive(Deserialize)]
pub struct ScriptProgram {
    pub(crate) statements: Vec<Statement>,
}

impl Evaluate for ScriptProgram {
    type Value = ();

    fn eval(&self, ctx: &mut Context<'_>) -> Fallible<Self::Value> {
        for stmt in &self.statements {
            stmt.eval(ctx)?;
        }
        Ok(())
    }
}

mod internal {
    use std::fmt::{self, Display};
    use std::slice;

    use serde::de::{Deserializer, SeqAccess, Visitor};

    use super::*;

    #[derive(Clone, Debug, Eq, Hash, PartialEq)]
    pub struct PackagePath(SmallVec<[Symbol; 4]>);

    impl<'a> IntoIterator for &'a PackagePath {
        type Item = &'a Symbol;
        type IntoIter = slice::Iter<'a, Symbol>;

        fn into_iter(self) -> Self::IntoIter {
            self.0.iter()
        }
    }

    impl<'de> Deserialize<'de> for PackagePath {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
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
                    let mut res = seq.size_hint().map(SmallVec::with_capacity).unwrap_or_default();
                    while let Some(i) = seq.next_element()? {
                        res.push(i);
                    }
                    Ok(res)
                }
            }
            deserializer.deserialize_seq(V).map(PackagePath)
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
