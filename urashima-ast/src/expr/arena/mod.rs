#[cfg(deserialize)]
mod impls;

use std::marker::PhantomData;

use urashima_util::{Arena, Index};

#[cfg(deserialize)]
use serde::de::{self, DeserializeSeed, Deserializer, Visitor};

use super::Expression;
use crate::{
    error::Fallible,
    parser::{Pairs, Parse, Rule},
};

pub type ExprArena = Arena<Expression>;
pub type ExprIndex = Index<Expression>;

impl Parse for ExprIndex {
    const RULE: Rule = Rule::expression;

    fn from_pairs(arena: &mut ExprArena, pairs: Pairs<'_>) -> Fallible<Self> {
        let expr = Expression::from_pairs(&mut *arena, pairs)?;
        Ok(arena.insert(expr))
    }
}

pub struct Alloc<'a, T>(&'a mut ExprArena, PhantomData<T>);

impl<'a, 'de, T> From<&'a mut ExprArena> for Alloc<'a, T> {
    fn from(arena: &'a mut ExprArena) -> Self {
        Alloc(arena, PhantomData)
    }
}

impl<'a, 'de, T> Alloc<'a, T> {
    pub fn arena(&mut self) -> &mut ExprArena {
        &mut self.0
    }

    pub fn borrow<U>(&mut self) -> Alloc<'_, U> {
        Alloc(&mut *self.0, PhantomData)
    }
}

/// If the missing field is of type `Option<T>` then treat is as `None`,
/// otherwise it is an error.
#[cfg(deserialize)]
pub fn missing_field<'de, V, E>(seed: V, field: &'static str) -> Result<V::Value, E>
where
    V: DeserializeSeed<'de>,
    E: de::Error,
{
    use serde::forward_to_deserialize_any;

    struct MissingFieldDeserializer<E>(&'static str, PhantomData<E>);

    impl<'de, E> Deserializer<'de> for MissingFieldDeserializer<E>
    where
        E: de::Error,
    {
        type Error = E;

        fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, E>
        where
            V: Visitor<'de>,
        {
            Err(de::Error::missing_field(self.0))
        }

        fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, E>
        where
            V: Visitor<'de>,
        {
            visitor.visit_none()
        }

        forward_to_deserialize_any! {
            bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
            bytes byte_buf unit unit_struct newtype_struct seq tuple
            tuple_struct map struct enum identifier ignored_any
        }
    }

    let deserializer = MissingFieldDeserializer(field, PhantomData);
    DeserializeSeed::deserialize(seed, deserializer)
}
