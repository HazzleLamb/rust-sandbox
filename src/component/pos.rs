use std::any::TypeId;

use nalgebra::Vector3;

use super::{ComponentMarker, Tomb};

static TOMB: StellarPos = StellarPos {
    pos: Vector3::new(0.0, 0.0, 0.0),
};

#[derive(Clone, Default)]
pub struct StellarPos {
    pub pos: Vector3<f64>,
}
impl StellarPos {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            pos: Vector3::new(x, y, z),
        }
    }

    pub fn load(pos: Vector3<f64>) -> Self {
        Self { pos }
    }
}

impl ComponentMarker for StellarPos {
    fn id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

impl Tomb for StellarPos {
    fn tomb() -> &'static StellarPos {
        &TOMB
    }
}
