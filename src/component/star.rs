use std::{
    any::TypeId,
    sync::{Arc, Mutex},
};

use rand::rngs::ThreadRng;
use rayon::iter::ParallelBridge;

use crate::{entity::EntityStore, heap::HeapElemId, world::World};

use super::{pos::StellarPos, ComponentMarker, Tomb};

static TOMB: Star = Star {};

#[derive(Default)]
pub struct Star {}

impl Star {
    pub fn new() -> Self {
        Self {}
    }
}

impl ComponentMarker for Star {
    fn id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

impl Tomb for Star {
    fn tomb() -> &'static Star {
        &TOMB
    }
}
