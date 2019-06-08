use std::iter::FromIterator;

use serde::Deserialize;
use serde_derive_urashima::DeserializeSeed;

use super::{Symbol, Variant};
use crate::arena::Index;

#[derive(Clone, Debug)]
pub struct Record {
    pub(crate) fields: Vec<Field>,
}

impl Record {
    pub fn unit() -> Self {
        Record { fields: Vec::new() }
    }
}

impl Default for Record {
    fn default() -> Self {
        Record::unit()
    }
}

impl FromIterator<(Key, Index<Variant>)> for Record {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (Key, Index<Variant>)>,
    {
        let fields = iter
            .into_iter()
            .map(|(k, value)| match k {
                Key::Index(key) => Field::Index { key, value },
                Key::Label(key) => Field::Label { key, value },
            })
            .collect();
        Record { fields }
    }
}

#[derive(Clone, Debug)]
pub enum Field {
    Index { key: usize, value: Index<Variant> },
    Label { key: Symbol, value: Index<Variant> },
}

#[derive(Clone, Eq, Debug, Deserialize, DeserializeSeed, Hash, Ord, PartialEq, PartialOrd)]
pub enum Key {
    Index(usize),
    Label(Symbol),
}
