use std::any::TypeId;

use crate::heap::TyId;

use super::{ComponentMarker, Tomb};

static TOMB: Planet = Planet {};

#[derive(Default)]
pub struct Planet {}

impl Planet {
    pub fn new() -> Self {
        Self {}
    }
}

impl ComponentMarker for Planet {}

impl TyId for Planet {
    fn id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

impl Tomb for Planet {
    fn tomb() -> &'static Planet {
        &TOMB
    }
}
