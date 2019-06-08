use serde::Deserialize;
use serde_derive_urashima::DeserializeSeed;
use smallvec::SmallVec;

use crate::{
    capsule::Capsule, data::Symbol, error::Fallible, eval::Evaluate, expr::ExprIndex,
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

#[derive(Clone, Debug, DeserializeSeed)]
pub struct PackageDep {
    pub(crate) path: PackagePath,
    pub(crate) imports: Vec<Symbol>,
}

#[derive(DeserializeSeed)]
struct Binding {
    name: Symbol,
    value: ExprIndex,
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
    use std::slice;

    use serde::de::{DeserializeSeed, Deserializer, SeqAccess, Visitor};

    use super::*;
    use crate::expr::Alloc;

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
