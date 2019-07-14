use core::fmt::{self, Display};
use core::iter::FromIterator;
use core::slice;

use smallvec::SmallVec;

use crate::symbol::Symbol;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PackagePath(pub(crate) SmallVec<[Symbol; 4]>);

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
