use std::{
    ops::Neg,
    sync::{Arc, Mutex},
};

use nalgebra::{Rotation3, UnitQuaternion, Vector3};
use rand::{rngs::ThreadRng, Rng};
use rayon::iter::{
    IntoParallelIterator, IntoParallelRefIterator, ParallelBridge, ParallelIterator,
};

use crate::{
    component::{
        self,
        orbit::{SelestialOrbit, SelestialOrbitDirectopn},
        pos::StellarPos,
        ComponentStore, Tomb,
    },
    heap::HeapElemId,
    world::World,
};

use super::EntityStore;

pub const SECONDS_IN_HOUR: u64 = 60 * 60;
pub const SECONDS_IN_DAY: u64 = SECONDS_IN_HOUR * 24;
pub const SECONDS_IN_MONTH: u64 = SECONDS_IN_DAY * 30;

pub const METERS_IN_ONE_LY: u64 = AUS_IN_ONE_LY as u64 * METERS_IN_ONE_AU as u64;
pub const METERS_IN_ONE_AU: f64 = 149_597_870_691.0;
pub const AUS_IN_ONE_LY: f64 = 63241.0;

pub const N_STARS: usize = 50_000;
pub const N_PLANETS_PER_STAR: usize = 10;
pub const N_MOONS_PER_PLANET: usize = 5;

const ORBIT_AXIS_MAX_TILT: usize = 20;

const UNIVERSE_CENTER: Vector3<u64> = Vector3::new(u64::MAX / 2, u64::MAX / 2, u64::MAX / 2);

pub fn generate_stars(world: &mut World, n_stars: usize) -> Vec<HeapElemId<EntityStore>> {
    let world_ref = Arc::new(Mutex::new(world));

    let star_ids: Vec<_> = (0..n_stars)
        .par_bridge()
        .map(|_| world_ref.lock().unwrap().alloc_entity())
        .collect();

    star_ids
        .par_iter()
        .for_each_with(world_ref, |world_ref, star_id| {
            let mut rng = rand::thread_rng();
            let mut world_mutex = world_ref.lock().unwrap();

            world_mutex.add_component(star_id, StellarPos::new(0.0, 0.0, 0.0));
            world_mutex.add_component(star_id, component::star::Star {});
        });

    star_ids
}

pub fn generate_planets(
    world: &mut World,
    star_ids: Vec<HeapElemId<EntityStore>>,
    n_planets_per_star: usize,
) -> Vec<HeapElemId<EntityStore>> {
    let star_pos_ids: Vec<_> = star_ids
        .par_iter()
        .flat_map(|star_id| {
            (0..n_planets_per_star)
                .into_iter()
                .map(|_| *star_id)
                .par_bridge()
        })
        .map(|star_id| world.get_entity_component_key::<StellarPos>(&star_id))
        .collect();

    let world_ref = Arc::new(Mutex::new(world));

    let planet_with_star: Vec<_> = star_pos_ids
        .par_iter()
        .map_with(world_ref.clone(), |world_ref, star_pos| {
            let mut world_mutex = world_ref.lock().unwrap();

            let planet_id: HeapElemId<EntityStore> = world_mutex.alloc_entity();

            (planet_id, star_pos)
        })
        .collect();

    planet_with_star.par_iter().for_each_with(
        world_ref.clone(),
        |world_ref, (planet_id, start_pos_id)| {
            let mut rng = rand::thread_rng();
            let mut world_mutex = world_ref.lock().unwrap();
            let orbit = generate_planet_orbit(&mut rng, (*start_pos_id).clone());
            let planet_pos = generate_orbiting_pos(&mut rng, &world_mutex, &orbit);

            world_mutex.add_component(&planet_id, orbit);
            world_mutex.add_component(&planet_id, planet_pos);
            world_mutex.add_component(&planet_id, component::planet::Planet {});
        },
    );

    planet_with_star
        .into_par_iter()
        .map(|(planet_id, _)| planet_id)
        .collect()
}

pub fn generate_moons(
    world: &mut World,
    planet_ids: Vec<HeapElemId<EntityStore>>,
    n_moons_per_planet: usize,
) {
    let planet_pos_ids: Vec<_> = planet_ids
        .par_iter()
        .flat_map(|planet_id| {
            (0..n_moons_per_planet)
                .into_iter()
                .map(|_| *planet_id)
                .par_bridge()
        })
        .map(|planet_id| world.get_entity_component_key::<StellarPos>(&planet_id))
        .collect();

    let world_ref = Arc::new(Mutex::new(world));

    let moon_with_star: Vec<_> = planet_pos_ids
        .par_iter()
        .map_with(world_ref.clone(), |world_ref, planet_pos_id| {
            let mut world_mutex = world_ref.lock().unwrap();

            let moon_id: HeapElemId<EntityStore> = world_mutex.alloc_entity();

            (moon_id, planet_pos_id)
        })
        .collect();

    moon_with_star.par_iter().for_each_with(
        world_ref.clone(),
        |world_ref, (planet_id, planet_pos_id)| {
            let mut rng = rand::thread_rng();
            let mut world_mutex = world_ref.lock().unwrap();
            let orbit = generate_moon_orbit(&mut rng, (*planet_pos_id).clone());
            let moon_orbit = generate_orbiting_pos(&mut rng, &world_mutex, &orbit);

            world_mutex.add_component(&planet_id, orbit);
            world_mutex.add_component(&planet_id, moon_orbit);
            world_mutex.add_component(&planet_id, component::planet::Planet {});
        },
    );
}

fn generate_planet_orbit(
    rng: &mut ThreadRng,
    star_pos_id: HeapElemId<ComponentStore>,
) -> SelestialOrbit {
    SelestialOrbit::load(
        star_pos_id,
        generate_orbit_tilt(rng),
        rng.gen_range(METERS_IN_ONE_AU..(METERS_IN_ONE_AU * 200.0)),
        rng.gen_range((3 * SECONDS_IN_MONTH)..(36 * SECONDS_IN_MONTH)),
        SelestialOrbitDirectopn::random(rng.gen_range(0..100)),
        0,
    )
}

fn generate_orbit_tilt(rng: &mut ThreadRng) -> Rotation3<f64> {
    let pos_angle = ORBIT_AXIS_MAX_TILT as f64;

    let roll = rng.gen_range((pos_angle.neg())..pos_angle);
    let pitch = rng.gen_range((pos_angle.neg())..pos_angle);
    let yaw = rng.gen_range((pos_angle.neg())..pos_angle);

    Rotation3::from_euler_angles(roll, pitch, yaw)
}

fn generate_moon_orbit(
    rng: &mut ThreadRng,
    planet_pos_id: HeapElemId<ComponentStore>,
) -> SelestialOrbit {
    SelestialOrbit::load(
        planet_pos_id,
        generate_orbit_tilt(rng),
        rng.gen_range(100_000_000.0 as f64..100_000_000_000.0 as f64),
        rng.gen_range((3 * SECONDS_IN_HOUR)..(36 * SECONDS_IN_MONTH)),
        SelestialOrbitDirectopn::random(rng.gen_range(0..100)),
        0,
    )
}

fn generate_orbiting_pos(rng: &mut ThreadRng, world: &World, orbit: &SelestialOrbit) -> StellarPos {
    let root_pos = orbit
        .tilt
        .transform_vector(&Vector3::new(orbit.radius, 0.0, 0.0));

    let host_pos = world.component::<StellarPos>(&orbit.center_component);
    let offset_pos = root_pos + host_pos.pos;

    StellarPos::load(offset_pos)
}
