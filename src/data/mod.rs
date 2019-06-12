pub mod function;
pub mod record;
pub mod variant;

mod internal {
    use serde::de::{Deserialize, DeserializeSeed, Deserializer};

    use crate::expr::Alloc;

    use naru_symbol::{Symbol};

    impl<'a, 'de> DeserializeSeed<'de> for Alloc<'a, Symbol> {
        type Value = Symbol;

        fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            Deserialize::deserialize(deserializer)
        }
    }
}

pub use naru_symbol::{Symbol, symbol};
pub use self::{function::Function, record::Record, variant::Variant};

#[cfg(test)]
mod test {
    use std::mem;

    use super::{Symbol, symbol};

    #[test]
    fn symbol_size() {
        dbg!(mem::size_of::<Symbol>());
        unsafe {
            dbg!(mem::transmute::<_, [u8; 8]>(dbg!(symbol!("true"))));
            dbg!(mem::transmute::<_, [u8; 8]>(dbg!(Symbol::from("true"))));
            dbg!(mem::transmute::<_, [u8; 8]>(dbg!(Symbol::from("asdf"))));
            dbg!(mem::transmute::<_, [u8; 8]>(dbg!(Symbol::from(
                "the quick fox jumps over the lazy brown dog"
            ))));
        }
    }
}
