pub mod selestial;

use crate::heap::Heap;

pub struct Entity {}

pub struct EntityStore {
    store: Heap<EntityStore, Entity>,
}
impl EntityStore {
    pub(crate) fn new() -> Self {
        Self { store: Heap::new() }
    }

    pub(crate) fn alloc(&mut self) -> crate::heap::HeapElemId<EntityStore> {
        let entity_key = self.store.alloc();
        self.store.replace(&entity_key, Entity {});

        entity_key
    }
}
