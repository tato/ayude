use std::collections::HashMap;
use std::marker::PhantomData;
use std::hash::{Hash, Hasher};

pub struct Id<T>(u64, PhantomData<T>);

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        Self(self.0, PhantomData)
    }
}
impl<T> Copy for Id<T> { }
impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<T> Eq for Id<T> { }
impl<T> Hash for Id<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

pub struct Catalog<T> {
    items: HashMap<Id<T>, T>,
    counter: u64,
}

impl<T> Catalog<T>  {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            counter: 0,
        }
    }

    pub fn get(&self, id: Id<T>) -> Option<&T> {
        self.items.get(&id)
    }

    pub fn add(&mut self, it: T) -> Id<T> {
        let id = Id(self.counter, PhantomData);
        self.items.insert(id, it);
        self.counter += 1;
        id
    }
}