use std::iter::FromIterator;

use urashima_util::Index;

use super::{Symbol, Variant};

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

impl FromIterator<(Symbol, Index<Variant>)> for Record {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (Symbol, Index<Variant>)>,
    {
        let fields = iter
            .into_iter()
            .map(|(label, value)| Field { label, value })
            .collect();
        Record { fields }
    }
}

#[derive(Clone, Debug)]
pub struct Field {
    label: Symbol,
    value: Index<Variant>,
}
