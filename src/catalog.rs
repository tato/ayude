use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

pub struct Catalog<T> { 
    items: HashMap<Id<T>, T>,
    counter: u64,
}

impl<T> std::iter::FromIterator<T> for Catalog<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut c = Self::new();
        for i in iter {
            c.add(i);
        }
        c
    }
}

impl<T> Catalog<T> {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            counter: 0,
        }
    }
 
    pub fn get(&self, id: Id<T>) -> Option<&T> {
        self.items.get(&id)
    }

    pub fn get_opt(&self, id: Option<Id<T>>) -> Option<&T> {
        id.and_then(|it| self.items.get(&it))
    }

    pub fn get_mut(&mut self, id: Id<T>) -> Option<&mut T> {
        self.items.get_mut(&id)
    }

    pub fn add(&mut self, it: T) -> Id<T> {
        let id = Id(self.counter, PhantomData);
        self.items.insert(id, it);
        self.counter += 1;
        id
    }

    pub fn iter(&self) -> std::collections::hash_map::Values<'_, Id<T>, T> {
        self.items.values()
    }

    pub fn iter_ids(&mut self) -> std::collections::hash_map::Keys<'_, Id<T>, T> {
        self.items.keys()
    }

    pub fn iter_with_id(&mut self) -> std::collections::hash_map::IterMut<'_, Id<T>, T> {
        self.items.iter_mut()
    }
}


pub struct Id<T>(u64, PhantomData<T>);

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        Self(self.0, PhantomData)
    }
}
impl<T> Copy for Id<T> {}
impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<T> Eq for Id<T> {}
impl<T> Hash for Id<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}