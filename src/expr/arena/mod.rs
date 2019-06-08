mod impls;

use std::marker::PhantomData;
use std::ops::{Deref, DerefMut, Index, IndexMut};

use serde::de::{self, DeserializeSeed, Deserializer, Visitor};

use super::Expression;
use crate::{
    capsule::Capsule,
    data::Variant,
    error::{ErrorKind, Fallible},
    eval::Evaluate,
};

#[derive(Debug)]
pub struct ExprArena(generational_arena::Arena<Expression>);

#[derive(Clone, Copy, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct ExprIndex(generational_arena::Index);

pub(crate) struct Alloc<'a, T>(&'a mut ExprArena, PhantomData<T>);

impl ExprArena {
    pub fn new() -> ExprArena {
        ExprArena(generational_arena::Arena::new())
    }

    pub fn with_capacity(n: usize) -> ExprArena {
        ExprArena(generational_arena::Arena::with_capacity(n))
    }

    pub(crate) fn alloc<'a, 'de, T>(&'a mut self) -> Alloc<'a, T> {
        Alloc(self, PhantomData)
    }
}

impl Deref for ExprArena {
    type Target = generational_arena::Arena<Expression>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ExprArena {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Index<ExprIndex> for ExprArena {
    type Output = Expression;

    fn index(&self, index: ExprIndex) -> &Self::Output {
        self.0.index(index.0)
    }
}

impl IndexMut<ExprIndex> for ExprArena {
    fn index_mut(&mut self, index: ExprIndex) -> &mut Self::Output {
        self.0.index_mut(index.0)
    }
}

impl Evaluate for ExprIndex {
    type Value = Variant;

    fn eval(&self, ctx: &mut Capsule) -> Fallible<Self::Value> {
        let expr = ctx
            .expr_arena
            .get(self.0)
            .ok_or_else(|| ErrorKind::Runtime)?
            .clone();
        expr.eval(ctx)
    }
}

impl<'a, 'de, T> From<&'a mut ExprArena> for Alloc<'a, T> {
    fn from(arena: &'a mut ExprArena) -> Self {
        arena.alloc()
    }
}

impl<'a, 'de, T> Alloc<'a, T> {
    pub(crate) fn arena(&mut self) -> &mut ExprArena {
        &mut *self.0
    }

    pub(crate) fn borrow<'b, U>(&'b mut self) -> Alloc<'b, U> {
        Alloc(self.arena(), PhantomData)
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
