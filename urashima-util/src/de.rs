use core::fmt;

use serde_state::de::{Deserialize, DeserializeState, Deserializer, SeqAccess, Visitor};
use smallvec::SmallVec;

use crate::{
    arena::{Arena, Index},
    pkg::PackagePath,
    symbol::Symbol,
};

impl<'de> Deserialize<'de> for PackagePath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct V;
        impl<'a> Visitor<'a> for V {
            type Value = SmallVec<[Symbol; 4]>;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("an array of strings")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'a>,
            {
                let mut res = seq
                    .size_hint()
                    .map(SmallVec::with_capacity)
                    .unwrap_or_default();
                while let Some(i) = seq.next_element()? {
                    res.push(i);
                }
                Ok(res)
            }
        }
        deserializer.deserialize_seq(V).map(PackagePath)
    }
}

impl<'de, T> DeserializeState<'de, Arena<T>> for Index<T>
where
    T: DeserializeState<'de, Arena<T>>,
{
    fn deserialize_state<D>(seed: &mut Arena<T>, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expr = DeserializeState::deserialize_state(seed, deserializer)?;
        Ok(seed.insert(expr))
    }
}

impl<'de, T> DeserializeState<'de, Arena<T>> for Symbol
where
    T: DeserializeState<'de, Arena<T>>,
{
    fn deserialize_state<D>(_seed: &mut Arena<T>, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Deserialize::deserialize(deserializer)
    }
}
