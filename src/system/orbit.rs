use std::{any::TypeId, collections::HashMap};

use nalgebra::Vector3;
use rayon::iter::IntoParallelRefIterator;

use crate::{
    component::{self, ComponentMarker, ComponentType, OrbitComponent, Pos, Tomb},
    state::State,
    EMPTY_VEC,
};

use super::SystemJob;

pub struct Orbit;

impl Default for Orbit {
    fn default() -> Self {
        Self {}
    }
}

pub struct OrbitScope {
    pub offset: Vector3<u64>,
}

impl SystemJob for Orbit {
    type AffectedComponent = component::Pos;

    type Scope = OrbitScope;

    fn filter_entities(entity_id: usize, state: &State) -> bool {
        state.entity_components[&entity_id].contains_key(&OrbitComponent::tomb().id())
    }

    fn gather_scope(entity_id: usize, state: &State) -> Self::Scope {
        let orbit_component_id = state.entity_components[&entity_id][&OrbitComponent::tomb().id()];
        let orbit_component: &OrbitComponent = state.components.get_as(orbit_component_id);
        let orbit_center_pos: &Pos = state.components.get_as(orbit_component.center_component);

        OrbitScope {
            offset: orbit_center_pos.pos.clone(),
        }
    }

    fn job(
        mut entity_component: HashMap<TypeId, Box<&mut dyn ComponentMarker>>,
        scope: rayon::Scope,
    ) {
        let orbit = entity_component[&OrbitComponent::tomb().id()]
            .as_any()
            .downcast_ref::<OrbitComponent>()
            .unwrap();

        let force = orbit.tilt.normalize() * orbit.radius as f32;
        let delta = Vector3::new(force.x as u64, force.y as u64, force.z as u64);

        let pos = entity_component
        .get_mut(&Pos::tomb().id())
        .unwrap()
        .as_any_mut()
        .downcast_mut::<Pos>()
        .unwrap();
        pos.pos = pos.pos + delta;
    }
}
