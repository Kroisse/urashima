use std::iter::FromIterator;

use serde::Deserialize;

use crate::environment::{Symbol, Value};

#[derive(Clone, Debug, Deserialize)]
pub struct Record {
    pub(crate) fields: Vec<Field>,
}

impl Default for Record {
    fn default() -> Self {
        Record { fields: Vec::default() }
    }
}

impl FromIterator<(Key, Value)> for Record {
    fn from_iter<T>(iter: T) -> Self where T: IntoIterator<Item = (Key, Value)> {
        let fields = iter.into_iter().map(|(k, value)| match k {
            Key::Index(key) => Field::Index { key, value },
            Key::Label(key) => Field::Label { key, value },
        }).collect();
        Record { fields }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub enum Field {
    Index { key: usize, value: Value },
    Label { key: Symbol, value: Value },
}

#[derive(Clone, Eq, Debug, Deserialize, Hash, Ord, PartialEq, PartialOrd)]
pub enum Key {
    Index(usize),
    Label(Symbol),
}
