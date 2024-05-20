use std::any::TypeId;


use super::{ComponentMarker, Tomb};

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
