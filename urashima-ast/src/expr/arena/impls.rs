use std::fmt;

use naru_symbol::Symbol;
use serde::de::{self, Deserialize, DeserializeSeed, Deserializer, SeqAccess, Visitor};
use urashima_util::num::{Int, Nat};

use super::{super::Expression, Alloc, ExprArena, ExprIndex};

impl<'a, 'de> DeserializeSeed<'de> for Alloc<'a, ExprIndex> {
    /// The type produced by using this seed.
    type Value = ExprIndex;

    /// Equivalent to the more common `Deserialize::deserialize` method, except
    /// with some initial piece of data (the seed) passed in.
    fn deserialize<D>(mut self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expr = DeserializeSeed::deserialize(self.borrow::<Expression>(), deserializer)?;
        Ok(self.arena().insert(expr))
    }
}

impl<'a, 'de, T> DeserializeSeed<'de> for Alloc<'a, Option<T>>
where
    for<'b> Alloc<'b, T>: DeserializeSeed<'de, Value = T>,
{
    type Value = Option<T>;

    fn deserialize<D>(mut self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct OptionVisitor<'a, T> {
            seed: Alloc<'a, T>,
        }

        impl<'a, 'de, T> Visitor<'de> for OptionVisitor<'a, T>
        where
            for<'b> Alloc<'b, T>: DeserializeSeed<'de, Value = T>,
        {
            type Value = Option<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("option")
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(None)
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(None)
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                DeserializeSeed::deserialize(self.seed, deserializer).map(Some)
            }
        }

        let visitor = OptionVisitor {
            seed: self.borrow::<T>(),
        };
        deserializer.deserialize_option(visitor)
    }
}

impl<'a, 'de, T> DeserializeSeed<'de> for Alloc<'a, Vec<T>>
where
    for<'b> Alloc<'b, T>: DeserializeSeed<'de, Value = T>,
{
    type Value = Vec<T>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VecVisitor<'a, T> {
            seed: Alloc<'a, Vec<T>>,
        }

        impl<'a, 'de, T> Visitor<'de> for VecVisitor<'a, T>
        where
            for<'b> Alloc<'b, T>: DeserializeSeed<'de, Value = T>,
        {
            type Value = Vec<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a sequence")
            }

            fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut values = Vec::with_capacity(self::size_hint::cautious(seq.size_hint()));

                loop {
                    let seed = self.seed.borrow::<T>();
                    let next = seq.next_element_seed(seed)?;
                    if let Some(value) = next {
                        values.push(value);
                    } else {
                        break;
                    }
                }

                Ok(values)
            }
        }

        let visitor = VecVisitor { seed: self };
        deserializer.deserialize_seq(visitor)
    }
}

macro_rules! deserializeseed_tuple_impls {
    ($($len:tt => ($($n:tt $name:ident)+))+) => {
        $(
            impl<'a, 'de, $($name),+> DeserializeSeed<'de> for Alloc<'a, ($($name,)+)>
            where $(for<'b> Alloc<'b, $name>: DeserializeSeed<'de, Value = $name>),+
            {
                type Value = ($($name,)+);

                #[inline]
                fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    struct TupleVisitor<'a, $($name,)+> {
                        seed: Alloc<'a, ($($name,)+)>,
                    }

                    impl<'a, 'de, $($name),+> Visitor<'de> for TupleVisitor<'a, $($name,)+>
                    where $(for<'b> Alloc<'b, $name>: DeserializeSeed<'de, Value = $name>),+
                    {
                        type Value = ($($name,)+);

                        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                            formatter.write_str(concat!("a tuple of size ", $len))
                        }

                        #[inline]
                        #[allow(non_snake_case)]
                        fn visit_seq<A>(mut self, mut seq: A) -> Result<Self::Value, A::Error>
                        where
                            A: SeqAccess<'de>,
                        {
                            $(
                                let $name = match seq.next_element_seed(self.seed.borrow::<$name>())? {
                                    Some(value) => value,
                                    None => return Err(de::Error::invalid_length($n, &self)),
                                };
                            )+

                            Ok(($($name,)+))
                        }
                    }

                    deserializer.deserialize_tuple($len, TupleVisitor { seed: self })
                }
            }
        )+
    }
}

deserializeseed_tuple_impls! {
    1  => (0 T0)
    2  => (0 T0 1 T1)
    3  => (0 T0 1 T1 2 T2)
    4  => (0 T0 1 T1 2 T2 3 T3)
    5  => (0 T0 1 T1 2 T2 3 T3 4 T4)
    6  => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5)
    7  => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6)
    8  => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7)
    9  => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8)
    10 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9)
    11 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10)
    12 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11)
    13 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12)
    14 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13)
    15 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14)
    16 => (0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15)
}

macro_rules! deserializeseed_forwarded_impls {
    (@impl $t:ty) => {
        impl<'a, 'de> DeserializeSeed<'de> for Alloc<'a, $t> {
            type Value = $t;

            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>
            {
                Deserialize::deserialize(deserializer)
            }
        }
    };
    ($($t:ty)*) => { $( deserializeseed_forwarded_impls!(@impl $t); )* };
}

deserializeseed_forwarded_impls! {
    bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 isize usize Int Nat String Symbol
}

mod size_hint {
    pub(crate) fn cautious(s: Option<usize>) -> usize {
        s.unwrap_or(16)
    }
}
