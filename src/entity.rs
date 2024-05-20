pub mod selestial;

use crate::heap::Heap;

pub struct Entity {}

pub struct EntityStore {
    store: Heap<EntityStore, Entity>,
}
impl EntityStore {
    pub fn new() -> Self {
        Self { store: Heap::new() }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self { store: Heap::with_capacity(cap)}
    }

    pub fn alloc(&mut self) -> crate::heap::HeapElemId<EntityStore> {
        let entity_key = self.store.alloc();
        self.store.replace(&entity_key, Entity {});

        entity_key
    }
}
