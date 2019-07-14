use core::marker::PhantomData;
use core::ops;

#[derive(Clone, Debug)]
pub struct Arena<T>(generational_arena::Arena<T>);

#[derive(Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct Index<T>(generational_arena::Index, PhantomData<T>);

impl<T> Clone for Index<T> {
    fn clone(&self) -> Self {
        Index(self.0, PhantomData)
    }
}

impl<T> Copy for Index<T> {}

#[allow(dead_code)]
impl<T> Arena<T> {
    pub fn new() -> Self {
        Arena(generational_arena::Arena::new())
    }

    pub fn with_capacity(n: usize) -> Self {
        Arena(generational_arena::Arena::with_capacity(n))
    }

    pub fn try_insert(&mut self, value: T) -> Result<Index<T>, T> {
        self.0.try_insert(value).map(Index::from_raw)
    }

    pub fn insert(&mut self, value: T) -> Index<T> {
        Index::from_raw(self.0.insert(value))
    }

    pub fn get(&self, i: Index<T>) -> Option<&T> {
        self.0.get(i.0)
    }

    pub fn get_mut(&mut self, i: Index<T>) -> Option<&mut T> {
        self.0.get_mut(i.0)
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Arena(generational_arena::Arena::new())
    }
}

impl<T> ops::Deref for Arena<T> {
    type Target = generational_arena::Arena<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> ops::DerefMut for Arena<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> ops::Index<Index<T>> for Arena<T> {
    type Output = T;

    fn index(&self, index: Index<T>) -> &Self::Output {
        self.0.index(index.0)
    }
}

impl<T> ops::IndexMut<Index<T>> for Arena<T> {
    fn index_mut(&mut self, index: Index<T>) -> &mut Self::Output {
        self.0.index_mut(index.0)
    }
}

impl<T> Index<T> {
    fn from_raw(idx: generational_arena::Index) -> Self {
        Index(idx, PhantomData)
    }
}

#[cfg(test)]
mod test {
    use core::mem;

    use super::*;

    #[test]
    fn index_size() {
        assert!(mem::size_of::<Index<()>>() <= 16);
        // assert!(mem::size_of::<Index<crate::data::Variant>>() <= 16);
        // assert!(mem::size_of::<Index<crate::expr::Expression>>() <= 16);
    }
}
