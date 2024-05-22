pub mod moon;
pub mod orbit;
pub mod planet;
pub mod pos;
pub mod star;

use std::{
    any::{Any, TypeId}, ops::Deref
};

use ahash::AHashMap;

use crate::heap::{Heap, HeapElemId, TyId};

pub trait ComponentMarker: AToAny + TyId + Sync + Send {
}

pub trait AToAny {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: 'static> AToAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub trait Tomb {
    fn tomb() -> &'static Self;
}

impl TyId for Box<dyn ComponentMarker + 'static> {
    fn id(&self) -> TypeId {
        self.deref().id()
    }
}

pub struct ComponentStore {
    store: Heap<ComponentStore, Box<dyn ComponentMarker>>,
    ty_lookup: AHashMap<HeapElemId<ComponentStore>, TypeId>,
}
impl ComponentStore {
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            store: Heap::with_capacity(cap),
            ty_lookup: AHashMap::with_capacity(cap)
        }
    }

    pub fn alloc<T: ComponentMarker + Tomb + 'static>(&mut self) -> HeapElemId<ComponentStore> {
        let ty_id = T::tomb().id();
        let component_key = self.store.alloc();
        self.ty_lookup.insert(component_key, ty_id);

        component_key
    }

    pub fn alloc_n<T: ComponentMarker + Tomb + 'static>(&mut self, n: usize) -> Vec<HeapElemId<ComponentStore>> {
        let ty_id = T::tomb().id();
        let component_ids = self.store.alloc_n(n);

        for cocomponent_id in &component_ids {
            self.ty_lookup.insert(*cocomponent_id, ty_id);
        }

        component_ids
    }

    pub fn put<T: ComponentMarker + 'static>(
        &mut self,
        component_id: &HeapElemId<ComponentStore>,
        component: Box<T>,
    ) {
        self.store.replace(&component_id, component);
    }

    pub fn get_type_id(&self, component_id: &HeapElemId<ComponentStore>) -> TypeId {
        self.ty_lookup[&component_id]
    }

    pub fn get_as<T: ComponentMarker + Tomb + 'static>(
        &self,
        component_id: &HeapElemId<ComponentStore>,
    ) -> &T {
        let data = self.store.get(component_id).deref().as_any();

        if !data.is::<T>() {
            let t_type_id = T::tomb().id();
            let data_type_id = data.type_id();
            println!(
                "IS ERROR: data of type {:?} is actualy not {:?}",
                data_type_id, t_type_id
            );
            panic!()
        }

        data.downcast_ref::<T>().unwrap()
    }
}
