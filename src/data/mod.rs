pub mod function;
pub mod record;
pub mod variant;

#[macro_use]
mod internal {
    use serde::de::{Deserialize, DeserializeSeed, Deserializer};

    use crate::expr::Alloc;

    include!(concat!(env!("OUT_DIR"), "/symbol.rs"));

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

pub use self::{function::Function, internal::Symbol, record::Record, variant::Variant};

#[cfg(test)]
mod test {
    use std::mem;

    use super::Symbol;

    #[test]
    fn symbol_size() {
        dbg!(mem::size_of::<Symbol>());
        unsafe {
            dbg!(mem::transmute::<_, [u8; 8]>(symbol!("true")));
            dbg!(mem::transmute::<_, [u8; 8]>(Symbol::from("asdf")));
            dbg!(mem::transmute::<_, [u8; 8]>(Symbol::from(
                "the quick fox jumps over the lazy brown dog"
            )));
        }
    }
}
