pub mod component;
pub mod state;
pub mod system;

use std::{
    any::TypeId,
    arch::x86_64::{__m128, _mm_extract_ps, _mm_rsqrt_ps, _mm_set_ps},
    collections::{HashMap, HashSet},
    sync::atomic::{AtomicI64, AtomicU64, AtomicUsize},
};

use component::{Component, ComponentMarker, ComponentType, OrbitComponent, Pos};
use la_arena::Arena;
use nalgebra::Vector3;
use rand::{rngs::ThreadRng, Rng};
use rayon::iter::{
    IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};
use state::State;
use strum::IntoEnumIterator;
use system::SystemJob;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::format::FmtSpan;

use crate::{
    component::Tomb,
    system::orbit::{Orbit, OrbitScope},
};

const METERS_IN_ONE_AU: u64 = 149_597_870_691;
const AUS_IN_ONE_LY: u64 = 63241;
const N_ENTITIES: usize = 1_000_000;
const N_SYSTEMS: usize = 3_000;
const N_PLANETS_PER_SYSTEM: usize = 10;
const N_MOONS_PER_PLANET: usize = 5;

static EMPTY_VEC: Vec<usize> = vec![];

fn main() {
    fmt::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .with_target(false)
        .with_level(false)
        .init();

    println!(
        "u64:MAX is {} AU ({} LY)",
        u64::MAX / METERS_IN_ONE_AU,
        (u64::MAX / METERS_IN_ONE_AU) / AUS_IN_ONE_LY
    );

    rayon::ThreadPoolBuilder::new()
        .num_threads(6)
        .build_global()
        .unwrap();

    let mut rng = rand::thread_rng();
    let mut state = generate_entities(&mut rng);

    let affected_entities: Vec<usize> = state
        .entities
        .par_iter()
        .enumerate()
        .filter(|(entity_id, _)| Orbit::filter_entities(*entity_id, &state))
        .map(|(entity_id, _)| entity_id)
        .collect();

    let scopes: Vec<(usize, OrbitScope)> = affected_entities
        .par_iter()
        .map(|entity_id| (*entity_id, Orbit::gather_scope(*entity_id, &state)))
        .map(|entity_id, scope| {
            let entity_components = {
                let component_ids = state.entity_components[&entity_id];
            }
        })
        .collect();

    let rich_scopes = affected_entities.par_iter().map(|entity_id| {}).collect();
}

#[tracing::instrument(skip_all)]
fn generate_entities(rng: &mut ThreadRng) -> State {
    let mut state = State::new();

    for _ in 0..N_SYSTEMS {
        let system_id = state.create_entity();
        let system_pos = state.add_component(
            system_id,
            Pos::new(rand_u64(rng), rand_u64(rng), rand_u64(rng)),
        );

        for _ in 0..N_PLANETS_PER_SYSTEM {
            let planet_id = state.create_entity();

            let planet_pos = state.add_component(
                planet_id,
                Pos::new(rand_u64(rng), rand_u64(rng), rand_u64(rng)),
            );

            let planet_orbit = OrbitComponent::new(
                system_pos,
                Vector3::new(rand_angle(rng), rand_angle(rng), rand_angle(rng)),
                rand_thrust(rng) as u64,
                rand_thrust(rng) as u64,
            );

            state.add_component(planet_id, planet_orbit);

            for _ in 0..N_MOONS_PER_PLANET {
                let moon_id: usize = state.create_entity();
                let moon_orbit = OrbitComponent::new(
                    planet_pos,
                    Vector3::new(rand_angle(rng), rand_angle(rng), rand_angle(rng)),
                    rand_thrust(rng) as u64,
                    rand_thrust(rng) as u64,
                );

                state.add_component(moon_id, moon_orbit);
            }
        }
    }

    state
}

#[tracing::instrument(skip_all)]
fn move_entities(entities: &mut Vec<()>) {
    entities.par_iter_mut().for_each(|entity| {
        // let force = entity.thrust_dir.normalize() * entity.thrust_power;

        // entity.pos = entity.pos + Vector3::new(force.x as u64, force.y as u64, force.z as u64);
    });
}

fn rand_u64(rng: &mut ThreadRng) -> u64 {
    let quoter_au = u64::MAX / 4;
    rng.gen_range(quoter_au..3 * quoter_au)
}

fn rand_thrust(rng: &mut ThreadRng) -> f32 {
    let one_au = 149_598_000_000.0;
    rng.gen_range(one_au..20.0 * one_au)
}

fn rand_angle(rng: &mut ThreadRng) -> f32 {
    rng.gen_range(0.0..=360.0)
}
