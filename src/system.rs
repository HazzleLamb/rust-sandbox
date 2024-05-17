pub mod orbit;

use std::{any::TypeId, collections::HashMap};

use rayon::Scope;
use strum_macros::EnumIter;

use crate::{component::{Component, ComponentMarker}, state::State};

use self::orbit::Orbit;

#[derive(EnumIter)]
pub enum System {
    Orbit(Orbit),
}

pub struct SystemScope {
    pub target: usize,
    pub involved: Vec<usize>
}

pub struct SystemScopeInvolvy {
    pub components: Vec<usize>
}

pub trait SystemJob {
    type AffectedComponent;
    type Scope;

    fn filter_entities(entity_id: usize, state: &State) -> bool;
    fn gather_scope(entity_id: usize, state: &State) -> Self::Scope;
    fn job(affected_component: HashMap<TypeId, Box<&mut dyn ComponentMarker>>, scope: Scope);
}