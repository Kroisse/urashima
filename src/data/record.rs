use std::collections::HashMap;
use std::fmt::{self, Display};
use std::iter::FromIterator;

use urashima_util::Index;

use super::{Symbol, Variant};

#[derive(Clone)]
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

#[derive(Clone)]
pub struct Field {
    label: Symbol,
    value: Index<Variant>,
}

pub struct RecordType {
    length: usize,
    labels: HashMap<Symbol, usize>,
}

impl RecordType {
    pub(crate) fn to_index(&self, label: Symbol) -> Option<usize> {
        self.labels.get(&label).copied()
    }
}

impl Display for RecordType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut fields = vec![None; self.length];
        for (label, idx) in &self.labels {
            fields[*idx] = Some(label);
        }
        let mut fields = fields.into_iter();
        f.write_str("(")?;
        if let Some(label) = fields.next() {
            if let Some(label) = label {
                write!(f, "{}: ", label)?;
            }
            write!(f, "any")?;
            if fields.len() == 0 {
                f.write_str(",")?;
            }
        }
        for label in fields {
            f.write_str(", ")?;
            if let Some(label) = label {
                write!(f, "{}: ", label)?;
            }
            write!(f, "any")?;
        }
        f.write_str(")")?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn record_type_unit_display() {
        let rec = RecordType {
            length: 0,
            labels: HashMap::new(),
        };
        assert_eq!("()", rec.to_string());
    }

    #[test]
    fn record_type_single_display() {
        let rec = RecordType {
            length: 1,
            labels: HashMap::new(),
        };
        assert_eq!("(any,)", rec.to_string());
    }

    #[test]
    fn record_type_single_labeled_display() {
        let mut rec = RecordType {
            length: 1,
            labels: HashMap::new(),
        };
        rec.labels.insert("foo".into(), 0);
        assert_eq!("(foo: any,)", rec.to_string());
    }

    #[test]
    fn record_type_display() {
        let rec = RecordType {
            length: 3,
            labels: HashMap::new(),
        };
        assert_eq!("(any, any, any)", rec.to_string());
    }

    #[test]
    fn record_type_display_2() {
        let mut rec = RecordType {
            length: 3,
            labels: HashMap::new(),
        };
        rec.labels.insert("x".into(), 0);
        rec.labels.insert("y".into(), 1);
        rec.labels.insert("z".into(), 2);
        assert_eq!("(x: any, y: any, z: any)", rec.to_string());
    }
}
