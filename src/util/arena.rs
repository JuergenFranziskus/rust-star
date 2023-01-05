use std::{
    any::type_name,
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    ops::{Index, IndexMut},
};

pub struct Arena<T> {
    items: Vec<T>,
}
impl<T> Arena<T> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn push(&mut self, item: T) -> ID<T> {
        let id = ID::new(self.len());
        self.items.push(item);
        id
    }
    pub fn next_id(&self) -> ID<T> {
        ID::new(self.len())
    }

    pub fn get(&self, id: ID<T>) -> Option<&T> {
        self.items.get(id.index)
    }
    pub fn get_mut(&mut self, id: ID<T>) -> Option<&mut T> {
        self.items.get_mut(id.index)
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter()
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.items.iter_mut()
    }

    pub fn find<F>(&self, f: F) -> Option<ID<T>>
    where
        F: Fn(&T) -> bool,
    {
        self.items
            .iter()
            .enumerate()
            .filter(|(_, t)| f(t))
            .next()
            .map(|(i, _)| ID::new(i))
    }
}
impl<T> Index<ID<T>> for Arena<T> {
    type Output = T;

    fn index(&self, index: ID<T>) -> &Self::Output {
        self.get(index).unwrap()
    }
}
impl<T> IndexMut<ID<T>> for Arena<T> {
    fn index_mut(&mut self, index: ID<T>) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}
impl<T> IntoIterator for Arena<T> {
    type IntoIter = std::vec::IntoIter<T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

pub struct ID<T> {
    index: usize,
    _phantom: PhantomData<T>,
}
impl<T> ID<T> {
    fn new(index: usize) -> Self {
        Self {
            index,
            _phantom: PhantomData,
        }
    }
}
impl<T> Clone for ID<T> {
    fn clone(&self) -> Self {
        ID {
            index: self.index,
            _phantom: PhantomData,
        }
    }
}
impl<T> Copy for ID<T> {}
impl<T> PartialEq for ID<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}
impl<T> Eq for ID<T> {}
impl<T> Hash for ID<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state)
    }
}
impl<T> Debug for ID<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ID<{}>({})", type_name::<T>(), self.index)
    }
}
