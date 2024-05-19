pub mod moon;
pub mod orbit;
pub mod planet;
pub mod pos;
pub mod star;

use std::{
    any::{Any, TypeId},
    ops::Deref,
};

use crate::heap::{Heap, HeapElemId};

pub trait ComponentMarker: AToAny + Sync + Send {
    fn id(&self) -> TypeId;
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

pub struct ComponentStore {
    store: Heap<ComponentStore, Box<dyn ComponentMarker>>,
}
impl ComponentStore {
    pub fn new() -> Self {
        Self { store: Heap::new() }
    }

    pub fn put<T: ComponentMarker + 'static>(
        &mut self,
        component: Box<T>,
    ) -> HeapElemId<ComponentStore> {
        let component_key = self.store.alloc();
        self.store.replace(&component_key, component);

        component_key
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
