pub mod component;
pub mod entity;
pub mod heap;
pub mod system;
pub mod world;

use std::{
    any::TypeId,
    arch::x86_64::{__m128, _mm_extract_ps, _mm_rsqrt_ps, _mm_set_ps},
    collections::{HashMap, HashSet},
    sync::atomic::{AtomicI64, AtomicU64, AtomicUsize},
};

use component::{orbit::SelestialOrbit, pos::StellarPos, ComponentMarker};
use entity::selestial::{
    generate_moons, generate_planets, generate_stars, METERS_IN_ONE_LY, N_MOONS_PER_PLANET,
    N_PLANETS_PER_STAR, N_STARS,
};
use la_arena::Arena;
use nalgebra::Vector3;
use rand::{rngs::ThreadRng, Rng};
use rayon::iter::{
    IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};
use strum::IntoEnumIterator;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::format::FmtSpan;
use world::World;

use crate::{component::Tomb, entity::selestial::METERS_IN_ONE_AU};

const N_ENTITIES: usize = 1_000_000;

fn main() {
    fmt::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .with_target(false)
        .with_level(false)
        .init();

    rayon::ThreadPoolBuilder::new()
        .num_threads(6)
        .build_global()
        .unwrap();
    let mut world = World::new();

    generate_stellar(&mut world);
}

#[tracing::instrument(skip_all)]
pub fn generate_stellar(world: &mut World) {
    println!(
        "Generating {} entities",
        N_STARS
            + (N_STARS * N_PLANETS_PER_STAR)
            + (N_STARS * N_PLANETS_PER_STAR * N_MOONS_PER_PLANET)
    );

    let generate_stars = generate_stars(world, N_STARS);
    let star_ids = generate_stars;
    let planet_ids = generate_planets(world, star_ids, N_PLANETS_PER_STAR);
    generate_moons(world, planet_ids, N_MOONS_PER_PLANET);
}

#[tracing::instrument(skip_all)]
fn move_entities(entities: &mut Vec<()>) {
    entities.par_iter_mut().for_each(|entity| {
        // let force = entity.thrust_dir.normalize() * entity.thrust_power;

        // entity.pos = entity.pos + Vector3::new(force.x as u64, force.y as u64, force.z as u64);
    });
}
