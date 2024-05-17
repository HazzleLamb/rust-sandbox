use std::{any::TypeId, collections::HashMap, sync::atomic::AtomicUsize};

use crate::component::ComponentMarker;

static COMPONENT_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub struct ComponentArena {
    data: HashMap<usize, Box<dyn ComponentMarker>>,
}

impl ComponentArena {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn push<T: ComponentMarker + 'static>(&mut self, component: T) -> usize {
        let id = COMPONENT_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.data.insert(id, Box::new(component));
        id
    }

    pub fn get_as<T: 'static>(&self, id: usize) -> &T {
        &*self.data[&id].as_any().downcast_ref::<T>().unwrap()
    }
}

pub struct State {
    pub entities: Vec<()>,
    pub components: ComponentArena,
    pub entities_with_component_map: HashMap<TypeId, Vec<usize>>,
    pub entity_components: HashMap<usize, HashMap<TypeId, usize>>,
}

impl State {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            components: ComponentArena::new(),
            entity_components: HashMap::new(),
            entities_with_component_map: HashMap::new(),
        }
    }

    pub fn create_entity(&mut self) -> usize {
        let entity_id = self.entities.len();

        self.entities.push(());
        self.entity_components.insert(entity_id, HashMap::new());

        entity_id
    }

    pub fn add_component<T: ComponentMarker>(&mut self, entity_id: usize, component: T) -> usize {
        let component_type_id = component.id();

        let curr_entity_components = self.entity_components.entry(entity_id).or_default();
        
        if let Some(component_id) = curr_entity_components.get(&component_type_id) {
            *component_id
        } else {
            let component_id = self.components.push(component);
            curr_entity_components.insert(component_type_id, component_id);
            self.entities_with_component_map.entry(component_type_id).or_default().push(entity_id);
            component_id
        }
    }
}
