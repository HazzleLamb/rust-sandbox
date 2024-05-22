pub mod selestial;

use std::any::TypeId;

use crate::heap::{Heap, HeapElemId, TyId};

#[derive(Clone, Copy)]
pub struct Entity {}

impl TyId for Entity {
    fn id(&self) -> std::any::TypeId {
        TypeId::of::<Self>()
    }
}

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

    pub fn alloc(&mut self) -> HeapElemId<EntityStore> {
        let entity_key = self.store.alloc();
        self.store.replace(&entity_key, Entity {});

        entity_key
    }

    pub fn alloc_n(&mut self, n: usize) -> Vec<HeapElemId<EntityStore>> {
        let keys = self.store.alloc_n(n);

        for key in &keys {
            self.store.replace(key, Entity {})
        }

        keys
    }
}
