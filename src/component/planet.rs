use std::any::TypeId;

use super::{ComponentMarker, Tomb};

static TOMB: Planet = Planet {};

#[derive(Default)]
pub struct Planet {}

impl Planet {
    pub fn new() -> Self {
        Self {}
    }
}

impl ComponentMarker for Planet {
    fn id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

impl Tomb for Planet {
    fn tomb() -> &'static Planet {
        &TOMB
    }
}
