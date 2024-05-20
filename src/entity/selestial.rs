use std::{
    cell::RefCell,
    ops::Neg,
    rc::Rc, sync::RwLock,
};

use rayon::prelude::*;

use nalgebra::{Rotation3, Vector3};
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{
    component::{
        self,
        moon::Moon,
        orbit::{SelestialOrbit, SelestialOrbitDirectopn},
        pos::SelestialPos,
        ComponentStore,
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

// const UNIVERSE_CENTER: Vector3<u64> = Vector3::new(u64::MAX / 2, u64::MAX / 2, u64::MAX / 2);

#[tracing::instrument(skip_all)]
pub fn generate_stars(world: Rc<RefCell<World>>, n_stars: usize) -> Vec<HeapElemId<EntityStore>> {
    let star_ids = alloc_stars(&world, n_stars);
    let star_with_components = alloc_star_components(&world, star_ids);
    let bound_star_ids = bind_star_components(&world, star_with_components);

    bound_star_ids.collect()
}

fn alloc_stars<'a>(
    world: &'a Rc<RefCell<World>>,
    n_stars: usize,
) -> impl Iterator<Item = HeapElemId<EntityStore>> + 'a {
    (0..n_stars).into_iter().map(|_| {
        let l_world_rc = &*Rc::clone(world);
        let mut world_mut = l_world_rc.borrow_mut();
        let star_id = world_mut.alloc_entity();

        star_id
    })
}

fn alloc_star_components<'a, T: Iterator<Item = HeapElemId<EntityStore>> + Sized + 'a>(
    world: &'a Rc<RefCell<World>>,
    stars: T,
) -> impl Iterator<
    Item = (
        HeapElemId<EntityStore>,
        HeapElemId<ComponentStore>,
        HeapElemId<ComponentStore>,
    ),
> + 'a {
    stars.map(move |star_id| {
        let l_world_rc = &*Rc::clone(world);
        let mut l_world_mut = l_world_rc.borrow_mut();
        let res = (
            star_id,
            l_world_mut
                .put_component(component::star::Star {}),
            l_world_mut
                .put_component(SelestialPos::new(0.0, 0.0, 0.0)),
        );

        res
    })
}

fn bind_star_components<
    'a,
    T: Iterator<
            Item = (
                HeapElemId<EntityStore>,
                HeapElemId<ComponentStore>,
                HeapElemId<ComponentStore>,
            ),
        > + Sized
        + 'a,
>(
    world: &'a Rc<RefCell<World>>,
    stars: T,
) -> impl Iterator<Item = HeapElemId<EntityStore>> + 'a {
    stars.map(|(star_id, mark_id, pos_id)| {
        let l_world_rc = &*Rc::clone(world);

        l_world_rc.borrow_mut().bind_component(star_id, mark_id);
        l_world_rc.borrow_mut().bind_component(star_id, pos_id);

        star_id
    })
}

#[tracing::instrument(skip_all)]
pub fn generate_planets(
    world: Rc<RefCell<World>>,
    star_ids: Vec<HeapElemId<EntityStore>>,
    n_planets_per_star: usize,
) -> Vec<HeapElemId<EntityStore>> {
    let world_rc = &*Rc::clone(&world);
    let world_ref = world_rc.borrow();
    let star_pos_ids: Vec<_> = world_ref.get_many_entities_compoents_key::<SelestialPos>(&star_ids);
    drop(world_ref);

    let planet_with_star = star_pos_ids
        .into_iter()
        .flat_map(|star_pos_id| std::iter::repeat(star_pos_id).take(n_planets_per_star))
        .map(|star_pos| {
            let l_world_rc = &*Rc::clone(&world);
            let mut world_mut = l_world_rc.borrow_mut();
            let planet_id: HeapElemId<EntityStore> = world_mut.alloc_entity();

            (planet_id, star_pos)
        });

    let mut rng = StdRng::from_seed([42; 32]);
    let planet_with_components = planet_with_star
        .map(
            |(planet_id, star_pos_id)| {
                let planet_orbit = generate_planet_orbit(&mut rng, star_pos_id);
                let planet_pos = generate_orbiting_pos(&world_rc.borrow(), &planet_orbit);

                (planet_id, planet_pos, planet_orbit)
            },
        );

    let processed_planets = planet_with_components.map(|(planet_id, planet_pos, planet_orbit)| {
        let l_world_rc = &*Rc::clone(&world);
        let mut l_world_mut = l_world_rc.borrow_mut();

        let marker_id = l_world_mut.put_component(component::planet::Planet {});
        let pos_id = l_world_mut.put_component(planet_pos);
        let orbit_id = l_world_mut.put_component(planet_orbit);

        l_world_mut.bind_component(planet_id, marker_id);
        l_world_mut.bind_component(planet_id, pos_id);
        l_world_mut.bind_component(planet_id, orbit_id);

        planet_id
    });

    processed_planets.collect()
}

#[tracing::instrument(skip_all)]
pub fn generate_moons(
    world: Rc<RefCell<World>>,
    planet_ids: Vec<HeapElemId<EntityStore>>,
    n_moons_per_planet: usize,
) {
    let planet_pos_ids = get_planet_pos_ids(&world.as_ref().borrow(), planet_ids).into_iter();

    let world_mut_ref: &mut World = &mut world.as_ref().borrow_mut();
    let world_mut_lock = RwLock::new(world_mut_ref);

    let moons_with_planet_pos = alloc_moons(&world_mut_lock, planet_pos_ids, n_moons_per_planet);
    let moons_with_components = populate_moon_components(&world_mut_lock, moons_with_planet_pos);

    let moons_with_component_ids = alloc_moon_components(&world_mut_lock, moons_with_components);
    let _: Vec<_> = bind_moon_components(&world_mut_lock, moons_with_component_ids).collect();
}

#[tracing::instrument(skip_all)]
fn get_planet_pos_ids(
    world: &World,
    planet_ids: Vec<HeapElemId<EntityStore>>,
) -> Vec<HeapElemId<ComponentStore>> {
    world.get_many_entities_compoents_key::<SelestialPos>(&planet_ids)
}

fn alloc_moons<'a, T: Iterator<Item = HeapElemId<ComponentStore>> + Send + 'a>(
    world: &'a RwLock<&'a mut World>,
    planet_pos_ids: T,
    n_moons_per_planet: usize,
) -> impl Iterator<Item = (HeapElemId<EntityStore>, HeapElemId<ComponentStore>)> + Send + 'a {
    planet_pos_ids
        .flat_map(move |planet_pos_id| std::iter::repeat(planet_pos_id).take(n_moons_per_planet))
        .map(|planet_pos_id| {
            let moon_id: HeapElemId<EntityStore> = world.write().unwrap().alloc_entity();

            (moon_id, planet_pos_id)
        })
}

fn populate_moon_components<'a, T: Iterator<Item = (HeapElemId<EntityStore>, HeapElemId<ComponentStore>)> + Send + 'a>(
    world: &'a RwLock<&'a mut World>,
    moons_with_planet_pos: T,
) -> impl Iterator<Item = (HeapElemId<EntityStore>, SelestialPos, SelestialOrbit)> + 'a {
    moons_with_planet_pos
        .map(
            |(moon_id, planet_pos_id)| {
                let moon_orbit = generate_moon_orbit(&mut StdRng::from_seed([42; 32]), planet_pos_id);
                let moon_pos = generate_orbiting_pos(&world.read().unwrap(), &moon_orbit);

                (moon_id, moon_pos, moon_orbit)
            },
        ).collect::<Vec<_>>().into_iter()
}

fn alloc_moon_components<'a, T: Iterator<Item = (HeapElemId<EntityStore>, SelestialPos, SelestialOrbit)> + 'a>(
    world: &'a RwLock<&'a mut World>,
    moons_with_components: T,
) -> impl Iterator<Item = (
    HeapElemId<EntityStore>,
    HeapElemId<ComponentStore>,
    HeapElemId<ComponentStore>,
    HeapElemId<ComponentStore>,
)> + 'a{
    moons_with_components
        .map(|(moon_id, moon_pos, moon_orbit)| {
            let mut world_mut = world.write().unwrap();
            (
                moon_id,
                world_mut.put_component::<Moon>(Moon {}),
                world_mut.put_component::<SelestialPos>(moon_pos),
                world_mut.put_component::<SelestialOrbit>(moon_orbit),
            )
        })
}

fn bind_moon_components<'a, T: Iterator<Item = (
    HeapElemId<EntityStore>,
    HeapElemId<ComponentStore>,
    HeapElemId<ComponentStore>,
    HeapElemId<ComponentStore>,
)> + 'a>(
    world: &'a RwLock<&'a mut World>,
    moons_with_components: T,
) -> impl Iterator<Item = ()> + 'a {
    moons_with_components.map(|(moon_id, moon_marker_id, moon_pos_id, moon_orbit_id)| {
        let mut world_mut = world.write().unwrap();

        world_mut.bind_component(moon_id, moon_marker_id);
        world_mut.bind_component(moon_id, moon_pos_id);
        world_mut.bind_component(moon_id, moon_orbit_id);

        ()
    })
}

fn generate_planet_orbit(
    rng: &mut StdRng,
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

fn generate_orbit_tilt(rng: &mut StdRng) -> Rotation3<f64> {
    let pos_angle = ORBIT_AXIS_MAX_TILT as f64;

    let roll = rng.gen_range((pos_angle.neg())..pos_angle);
    let pitch = rng.gen_range((pos_angle.neg())..pos_angle);
    let yaw = rng.gen_range((pos_angle.neg())..pos_angle);

    Rotation3::from_euler_angles(roll, pitch, yaw)
}

fn generate_moon_orbit(
    rng: &mut StdRng,
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

fn generate_orbiting_pos(world: &World, orbit: &SelestialOrbit) -> SelestialPos {
    let root_pos = orbit
        .tilt
        .transform_vector(&Vector3::new(orbit.radius, 0.0, 0.0));

    let host_pos = world.component::<SelestialPos>(&orbit.center_component);
    let offset_pos = root_pos + host_pos.pos;

    SelestialPos::load(offset_pos)
}
