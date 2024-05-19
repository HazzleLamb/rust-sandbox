use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{atomic::AtomicUsize, Arc, Mutex},
    vec,
};

use rayon::{
    collections::hash_map::IterMut,
    iter::{IntoParallelRefMutIterator, ParallelIterator},
};

use crate::{
    component::{self, ComponentMarker, ComponentStore, Tomb},
    entity::{Entity, EntityStore},
    heap::{self, HeapElemId},
};

pub struct World {
    entities: EntityStore,
    components: ComponentStore,
    entity_component_map:
        HashMap<HeapElemId<EntityStore>, HashMap<TypeId, HeapElemId<ComponentStore>>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: EntityStore::new(),
            components: ComponentStore::new(),
            entity_component_map: HashMap::new(),
        }
    }

    pub fn alloc_entity(&mut self) -> HeapElemId<EntityStore> {
        let entity_key = self.entities.alloc();
        self.entity_component_map.insert(entity_key, HashMap::new());
        entity_key
    }

    pub(crate) fn add_component<T: ComponentMarker + 'static>(
        &mut self,
        entity_key: &HeapElemId<EntityStore>,
        component: T,
    ) -> HeapElemId<ComponentStore> {
        let component_ty_id = component.id();
        let component_key = self.components.put(Box::new(component));
        self.entity_component_map
            .entry(*entity_key)
            .or_default()
            .insert(component_ty_id, component_key);

        component_key
    }

    pub fn get_entity_component_key<T: ComponentMarker + Tomb + 'static>(
        &self,
        star_id: &HeapElemId<EntityStore>,
    ) -> HeapElemId<ComponentStore> {
        let component_ty: TypeId = T::tomb().id();
        self.entity_component_map
            .get(star_id)
            .unwrap()
            .get(&component_ty)
            .copied()
            .unwrap()
    }

    pub fn component<T: ComponentMarker + Tomb + 'static>(
        &self,
        component_id: &HeapElemId<ComponentStore>,
    ) -> &T {
        self.components.get_as::<T>(component_id)
    }
}
