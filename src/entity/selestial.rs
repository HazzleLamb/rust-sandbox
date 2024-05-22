use std::{
    cell::RefCell,
    ops::Neg,
    rc::Rc,
    time::{Duration, Instant},
};

use nalgebra::{Rotation3, Vector3};

use rayon::prelude::*;

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
pub const METERS_IN_ONE_AU_U64: u64 = 149_597_870_691;
pub const METERS_IN_ONE_AU: f64 = 149_597_870_691.0;
pub const AUS_IN_ONE_LY: f64 = 63241.0;

pub const N_STARS: usize = 50_000;
pub const N_PLANETS_PER_STAR: usize = 10;
pub const N_MOONS_PER_PLANET: usize = 5;

const ORBIT_AXIS_MAX_TILT: isize = 20;

// const UNIVERSE_CENTER: Vector3<u64> = Vector3::new(u64::MAX / 2, u64::MAX / 2, u64::MAX / 2);

#[tracing::instrument(skip_all)]
pub fn generate_stars(world: Rc<RefCell<World>>, n_stars: usize) -> Vec<HeapElemId<EntityStore>> {
    let star_ids = alloc_stars(&mut world.borrow_mut(), n_stars);
    let star_with_components = alloc_star_components(&world, star_ids);
    let bound_star_ids = bind_star_components(&world, star_with_components);

    bound_star_ids.collect()
}

fn alloc_stars(
    world: &mut World,
    n_stars: usize,
) -> impl Iterator<Item = HeapElemId<EntityStore>> + 'static {
    world.alloc_n_entities(n_stars).into_iter()
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
            l_world_mut.put_component(component::star::Star {}),
            l_world_mut.put_component(SelestialPos::new(0.0, 0.0, 0.0)),
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

    let planets_n = star_pos_ids.len() * n_planets_per_star;

    let planet_ids = world.borrow_mut().alloc_n_entities(planets_n);

    let planet_with_star = planet_ids.into_iter().zip(
        star_pos_ids
            .into_iter()
            .flat_map(|star_pos_id| std::iter::repeat(star_pos_id).take(n_planets_per_star)),
    );

    let planet_with_components = planet_with_star.map(|(planet_id, star_pos_id)| {
        let planet_orbit = generate_planet_orbit(star_pos_id);
        let planet_pos = generate_orbiting_pos(&world_rc.borrow(), &planet_orbit);

        (planet_id, planet_pos, planet_orbit)
    });

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

static mut ALLOC_MOONS_DURS: Vec<Duration> = Vec::new();
static mut POPULATE_MOON_COMPONENTS_DURS: Vec<Duration> = Vec::new();
static mut ALLOC_MOON_COMPONENTS_DURS: Vec<Duration> = Vec::new();
static mut STORE_MOON_COMPONENTS_DURS: Vec<Duration> = Vec::new();
static mut BIND_MOON_COMPONENTS_DURS: Vec<Duration> = Vec::new();

#[tracing::instrument(skip_all)]
pub fn generate_moons(
    world: Rc<RefCell<World>>,
    planet_ids: Vec<HeapElemId<EntityStore>>,
    n_moons_per_planet: usize,
) {
    let s1 = Instant::now();
    let planet_pos_ids = get_planet_pos_ids(&world.as_ref().borrow(), planet_ids);
    println!(
        "Get planets poses total dur: {} ms",
        s1.elapsed().as_millis()
    );

    let moons_with_planet_pos =
        alloc_moons(&mut world.borrow_mut(), planet_pos_ids, n_moons_per_planet);
    let world_read = world.borrow();
    let moons_with_components = generate_moon_components(&world_read, moons_with_planet_pos);
    drop(world_read);

    let n_moons = moons_with_components.len();
    let moons_with_component_ids = alloc_moon_components(&mut world.borrow_mut(), moons_with_components.into_iter(), n_moons);
    bind_moon_components(&mut world.borrow_mut(), moons_with_component_ids.into_iter());

    unsafe {
        print_dur(&ALLOC_MOONS_DURS, "Alloc moons");
        print_dur(&POPULATE_MOON_COMPONENTS_DURS, "Populate moon components");
        print_dur(&ALLOC_MOON_COMPONENTS_DURS, "Alloc moon components");
        print_dur(&STORE_MOON_COMPONENTS_DURS, "Store moon components");
        print_dur(&BIND_MOON_COMPONENTS_DURS, "Bind moon components");
    }
}

fn print_dur(durs: &[Duration], caption: &'static str) {
    let mut dur_total = Duration::from_micros(0);
    for dur in durs {
        dur_total += *dur;
    }

    println!("{} total duration: {} ms", caption, dur_total.as_millis())
}

#[tracing::instrument(skip_all)]
fn get_planet_pos_ids(
    world: &World,
    planet_ids: Vec<HeapElemId<EntityStore>>,
) -> Vec<HeapElemId<ComponentStore>> {
    world.get_many_entities_compoents_key::<SelestialPos>(&planet_ids)
}

fn alloc_moons(
    world: &mut World,
    planet_pos_ids: Vec<HeapElemId<ComponentStore>>,
    n_moons_per_planet: usize,
) -> impl Iterator<Item = (HeapElemId<EntityStore>, HeapElemId<ComponentStore>)> + 'static {
    let a = Instant::now();

    let n_moons = planet_pos_ids.len() * n_moons_per_planet;
    let moon_ids = world.alloc_n_entities(n_moons);
    unsafe { ALLOC_MOONS_DURS.push(a.elapsed()) };

    moon_ids.into_iter().zip(
        planet_pos_ids.into_iter().flat_map(move |planet_pos_id| {
            std::iter::repeat(planet_pos_id).take(n_moons_per_planet)
        }),
    )
}

fn generate_moon_components<
    T: Iterator<Item = (HeapElemId<EntityStore>, HeapElemId<ComponentStore>)> + Send + 'static,
>(
    world: &World,
    moons_with_planet_pos: T,
) -> Vec<(HeapElemId<EntityStore>, SelestialPos, SelestialOrbit)>{
    let a = Instant::now();

    let res = moons_with_planet_pos.collect::<Vec<_>>().par_iter()
        .map(|(moon_id, planet_pos_id)| {
            let moon_orbit = generate_moon_orbit(*planet_pos_id);
            let moon_pos = generate_orbiting_pos(&world, &moon_orbit);
            (*moon_id, moon_pos, moon_orbit)
        })
        .collect::<Vec<_>>();

    unsafe { POPULATE_MOON_COMPONENTS_DURS.push(a.elapsed()) };

    res
}

fn alloc_moon_components<
    T: Iterator<Item = (HeapElemId<EntityStore>, SelestialPos, SelestialOrbit)> + 'static,
>(
    world: &mut World,
    moons_with_components: T,
    n_moons: usize,
) -> Vec<(
        HeapElemId<EntityStore>,
        HeapElemId<ComponentStore>,
    )> {
    let a = Instant::now();

    let moon_marker_ids = world.alloc_n_components::<Moon>(n_moons);
    let pos_ids = world.alloc_n_components::<SelestialPos>(n_moons);
    let orbit_ids = world.alloc_n_components::<SelestialOrbit>(n_moons);

    unsafe { ALLOC_MOON_COMPONENTS_DURS.push(a.elapsed()) };

    let a = Instant::now();

    let mut res = Vec::with_capacity(n_moons);

    for (idx, (moon_id, moon_pos, moon_orbit)) in moons_with_components.enumerate() {
        let marker_id = moon_marker_ids[idx];
        let pos_id = pos_ids[idx];
        let orbit_id = orbit_ids[idx];

        world.replace_component(&marker_id, Moon {});
        world.replace_component(&pos_id, moon_pos);
        world.replace_component(&orbit_id, moon_orbit);

        res.push((moon_id, marker_id));
        res.push((moon_id, pos_id));
        res.push((moon_id, orbit_id));
    }

    unsafe { STORE_MOON_COMPONENTS_DURS.push(a.elapsed()) };

    res
}

fn bind_moon_components<
    'a,
    T: Iterator<
            Item = (
                HeapElemId<EntityStore>,
                HeapElemId<ComponentStore>,
            ),
        > + 'a,
>(
    world: &mut World,
    moons_with_components: T,
) {
    let a = Instant::now();

    world.bind_n_components(moons_with_components.collect::<Vec<_>>());
    
    unsafe { BIND_MOON_COMPONENTS_DURS.push(a.elapsed()) };
}

fn generate_planet_orbit(
    star_pos_id: HeapElemId<ComponentStore>,
) -> SelestialOrbit {
    SelestialOrbit::load(
        star_pos_id,
        generate_orbit_tilt(),
        fastrand::u64(METERS_IN_ONE_AU_U64..(METERS_IN_ONE_AU_U64 * 200)) as f64,
        fastrand::u64((3 * SECONDS_IN_MONTH)..(36 * SECONDS_IN_MONTH)),
        SelestialOrbitDirectopn::random(fastrand::usize(0..100)),
        0,
    )
}

fn generate_orbit_tilt() -> Rotation3<f64> {
    let roll = fastrand::isize((ORBIT_AXIS_MAX_TILT.neg())..ORBIT_AXIS_MAX_TILT) as f64;
    let pitch = fastrand::isize((ORBIT_AXIS_MAX_TILT.neg())..ORBIT_AXIS_MAX_TILT) as f64;
    let yaw = fastrand::isize((ORBIT_AXIS_MAX_TILT.neg())..ORBIT_AXIS_MAX_TILT) as f64;

    Rotation3::from_euler_angles(roll, pitch, yaw)
}

fn generate_moon_orbit(
    planet_pos_id: HeapElemId<ComponentStore>,
) -> SelestialOrbit {
    SelestialOrbit::load(
        planet_pos_id,
        generate_orbit_tilt(),
        fastrand::u64(100_000_000..100_000_000_000) as f64,
        fastrand::u64((3 * SECONDS_IN_HOUR)..(36 * SECONDS_IN_MONTH)),
        SelestialOrbitDirectopn::random(fastrand::usize(0..100)),
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
