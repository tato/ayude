use std::collections::HashMap;

pub struct Handle<T> {
    id: u64,
    _type_marker: std::marker::PhantomData<T>,
}
impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl<T> Eq for Handle<T> { }
impl<T> std::hash::Hash for Handle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for Handle<T> { }

pub struct Catalog<T> {
    items: HashMap<Handle<T>, T>,
    latest_handle: u64,
}

impl<T> Catalog<T> {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            latest_handle: 0
        }
    }
    pub fn add(&mut self, it: T) -> Handle<T> {
        self.latest_handle += 1;
        let handle = Handle {
            id: self.latest_handle,
            _type_marker: std::marker::PhantomData,
        };
        self.items.insert(handle, it);
        handle
    }
    pub fn get(&mut self, handle: Handle<T>) -> Option<&mut T> {
        self.items.get_mut(&handle)
    }
}