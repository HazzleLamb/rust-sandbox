use std::any::TypeId;

use nalgebra::Rotation3;
use once_cell::sync::Lazy;

use crate::heap::{impossible_heap_elem_id, HeapElemId};

use super::{ComponentMarker, ComponentStore, Tomb};

static TOMB: Lazy<SelestialOrbit> = Lazy::new(|| SelestialOrbit {
    center_component: impossible_heap_elem_id::<ComponentStore>(),
    tilt: Rotation3::identity(),
    radius: 0.0,
    period_secs: 0,
    direction: SelestialOrbitDirectopn::CW,
    cycle_secs: 0,
});

#[derive(Default)]
pub enum SelestialOrbitDirectopn {
    #[default]
    CW,
    CCW,
}
impl SelestialOrbitDirectopn {
    pub fn random(seed: usize) -> SelestialOrbitDirectopn {
        if seed % 2 == 0 {
            Self::CW
        } else {
            Self::CCW
        }
    }
}

#[derive(Default)]
pub struct SelestialOrbit {
    pub center_component: HeapElemId<ComponentStore>,
    pub tilt: Rotation3<f64>,
    pub radius: f64,
    pub period_secs: u64,
    pub direction: SelestialOrbitDirectopn,
    pub cycle_secs: u64,
}
impl SelestialOrbit {
    pub(crate) fn load(
        center_component: HeapElemId<ComponentStore>,
        tilt: Rotation3<f64>,
        radius: f64,
        period_secs: u64,
        direction: SelestialOrbitDirectopn,
        cycle_secs: u64,
    ) -> Self {
        Self {
            center_component,
            tilt,
            radius,
            period_secs,
            direction,
            cycle_secs,
        }
    }
}

impl ComponentMarker for SelestialOrbit {
    fn id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

impl Tomb for SelestialOrbit {
    fn tomb() -> &'static Self {
        &TOMB
    }
}
