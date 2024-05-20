use std::any::TypeId;

use super::{ComponentMarker, Tomb};

static TOMB: Moon = Moon {};

#[derive(Default)]
pub struct Moon {}

impl Moon {
    pub fn new() -> Self {
        Self {}
    }
}

impl ComponentMarker for Moon {
    fn id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

impl Tomb for Moon {
    fn tomb() -> &'static Moon {
        &TOMB
    }
}