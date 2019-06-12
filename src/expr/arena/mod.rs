mod impls;

use std::marker::PhantomData;

use serde::de::{self, DeserializeSeed, Deserializer, Visitor};

use super::Expression;
use crate::{
    arena::{Arena, Index},
    capsule::Capsule,
    data::Variant,
    error::{Error, Fallible},
    eval::Evaluate,
};

pub type ExprArena = Arena<Expression>;
pub type ExprIndex = Index<Expression>;

pub(crate) struct Alloc<'a, T>(&'a mut Capsule, PhantomData<T>);

impl Capsule {
    pub(crate) fn alloc<T>(&mut self) -> Alloc<'_, T> {
        Alloc(self, PhantomData)
    }
}

impl Evaluate for ExprIndex {
    type Value = Variant;

    fn eval(&self, ctx: &mut Capsule) -> Fallible<Self::Value> {
        let expr = ctx
            .expr_arena
            .get(*self)
            .ok_or_else(Error::runtime)?
            .clone();
        expr.eval(ctx)
    }
}

impl<'a, 'de, T> From<&'a mut Capsule> for Alloc<'a, T> {
    fn from(arena: &'a mut Capsule) -> Self {
        arena.alloc()
    }
}

impl<'a, 'de, T> Alloc<'a, T> {
    pub(crate) fn arena(&mut self) -> &mut ExprArena {
        &mut self.0.expr_arena
    }

    pub(crate) fn borrow<U>(&mut self) -> Alloc<'_, U> {
        Alloc(&mut *self.0, PhantomData)
    }
}

/// If the missing field is of type `Option<T>` then treat is as `None`,
/// otherwise it is an error.
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
