use std::{
    any::{Any, TypeId},
    default,
};

use nalgebra::Vector3;

use strum_macros::{AsRefStr, EnumDiscriminants, EnumIter, IntoStaticStr};

#[derive(EnumIter, IntoStaticStr, AsRefStr, EnumDiscriminants)]
#[strum_discriminants(name(ComponentType))]
#[strum_discriminants(derive(Hash))]
pub enum Component {
    Pos {
        pos: Vector3<u64>,
    },
    Orbit {
        center_component: usize,
        radius: u64,
        period_secs: u64,
    },
}

pub trait ComponentMarker: AToAny + Sync {
    fn id(&self) -> TypeId;
}

pub trait AToAny: 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: 'static> AToAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub trait Tomb {
    fn tomb() -> &'static Self;
}

#[derive(Default)]
pub struct Pos {
    pub pos: Vector3<u64>,
}
impl Pos {
    pub fn new(x: u64, y: u64, z: u64) -> Self {
        Self {
            pos: Vector3::new(x, y, z),
        }
    }
}

impl ComponentMarker for Pos {
    fn id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

impl Tomb for Pos {
    fn tomb() -> &'static Pos {
        static TOMB: Pos = Pos {
            pos: Vector3::new(0, 0, 0),
        };
        &TOMB
    }
}

#[derive(Default)]
pub enum OrbitDirection {
    #[default]
    CW,
    CCW,
}

#[derive(Default)]
pub struct OrbitComponent {
    pub center_component: usize,
    pub tilt: Vector3<f32>,
    pub radius: u64,
    pub period_secs: u64,
    pub direction: OrbitDirection,
}
impl OrbitComponent {
    pub(crate) fn new(
        center_component: usize,
        tilt: Vector3<f32>,
        radius: u64,
        period_secs: u64,
    ) -> Self {
        Self {
            center_component,
            tilt,
            radius,
            period_secs,
            direction: OrbitDirection::CW,
        }
    }
}

impl ComponentMarker for OrbitComponent {
    fn id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

impl Tomb for OrbitComponent {
    fn tomb() -> &'static Self {
        static TOMB: OrbitComponent = OrbitComponent {
            center_component: 0,
            tilt: Vector3::new(0.0, 0.0, 0.0),
            radius: 0,
            period_secs: 0,
            direction: OrbitDirection::CW,
        };
        &TOMB
    }
}
