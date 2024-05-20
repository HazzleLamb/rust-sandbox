use std::any::TypeId;

use nalgebra::Vector3;

use super::{ComponentMarker, Tomb};

static TOMB: SelestialPos = SelestialPos {
    pos: Vector3::new(0.0, 0.0, 0.0),
};

#[derive(Clone, Default, Debug)]
pub struct SelestialPos {
    pub pos: Vector3<f64>,
}
impl SelestialPos {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            pos: Vector3::new(x, y, z),
        }
    }

    pub fn load(pos: Vector3<f64>) -> Self {
        Self { pos }
    }
}

impl ComponentMarker for SelestialPos {
    fn id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

impl Tomb for SelestialPos {
    fn tomb() -> &'static SelestialPos {
        &TOMB
    }
}
