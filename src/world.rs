use std::any::TypeId;

use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rustc_hash::FxHashMap;

use crate::{
    component::{ComponentMarker, ComponentStore, Tomb},
    entity::EntityStore,
    heap::HeapElemId,
};

pub struct World {
    entities: EntityStore,
    components: ComponentStore,
    entity_component_map: FxHashMap<(HeapElemId<EntityStore>, TypeId), HeapElemId<ComponentStore>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: EntityStore::new(),
            components: ComponentStore::new(),
            entity_component_map: FxHashMap::default(),
        }
    }

    pub fn with_capacity(n_entity: usize, n_components: usize) -> Self {
        Self {
            entities: EntityStore::with_capacity(n_entity),
            components: ComponentStore::with_capacity(n_components),
            entity_component_map: FxHashMap::default(),
        }
    }

    pub fn alloc_entity(&mut self) -> HeapElemId<EntityStore> {
        let entity_key = self.entities.alloc();
        entity_key
    }

    pub fn put_component<T: ComponentMarker + Tomb + 'static>(
        &mut self,
        component: T
    ) -> HeapElemId<ComponentStore> {
        let component_id = self.components.alloc::<T>();
        self.components.put(&component_id, Box::new(component));

        component_id
    }

    pub fn bind_component(&mut self, entity_id: HeapElemId<EntityStore>, component_id: HeapElemId<ComponentStore>) {
        let component_ty_id = self.components.get_type_id(&component_id);

        self.entity_component_map
        .insert((entity_id, component_ty_id), component_id);
    }

    pub fn get_entity_component_key<T: ComponentMarker + Tomb + 'static>(
        &self,
        entity_id: &HeapElemId<EntityStore>,
    ) -> HeapElemId<ComponentStore> {
        let component_ty: TypeId = T::tomb().id();
        self.entity_component_map
            .get(&(*entity_id, component_ty))
            .copied()
            .unwrap()
    }

    pub fn get_many_entities_compoents_key<T: ComponentMarker + Tomb + 'static>(
        &self,
        entity_ids: &[HeapElemId<EntityStore>],
    ) -> Vec<HeapElemId<ComponentStore>> {
        let component_ty: TypeId = T::tomb().id();
        entity_ids
            .into_par_iter()
            .map(|&entity_id| {
                self.entity_component_map
                    .get(&(entity_id, component_ty))
                    .copied()
                    .unwrap()
            })
            .collect()
    }

    pub fn component<T: ComponentMarker + Tomb + 'static>(
        &self,
        component_id: &HeapElemId<ComponentStore>,
    ) -> &T {
        self.components.get_as::<T>(component_id)
    }
}
