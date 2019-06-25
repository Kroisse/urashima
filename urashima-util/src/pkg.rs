use std::fmt::{self, Display};
use std::iter::FromIterator;
use std::slice;

use serde::de::{Deserialize, Deserializer, SeqAccess, Visitor};
use smallvec::SmallVec;

use crate::symbol::Symbol;

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
