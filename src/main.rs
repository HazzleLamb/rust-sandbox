use std::{f64::consts::PI, mem, ops::Neg};

use nalgebra::{Rotation3, Vector3};

use tracing_subscriber::fmt::{self, format::FmtSpan};

struct Star {}
struct Planet {
    star_id: usize,
}
struct Moon {
    planet_id: usize,
}

struct Translation {
    pos: Vector3<f64>,
}

struct Orbit {
    pub tilt: Rotation3<f64>,
    pub radius: f64,
    pub period_secs: u64,
}

struct World {
    tick_number: u64,
    stars: Vec<Option<Star>>,
    planet: Vec<Option<Planet>>,
    moon: Vec<Option<Moon>>,
    translation: Vec<Option<Translation>>,
    orbit: Vec<Option<Orbit>>,
}

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

fn main() {
    fmt::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .with_target(false)
        .with_level(false)
        .init();

    // Используем функции, чтобы было удобно мерять время. Не выкабениваемся
    let mut world = generate_world();

    let n_entities = world.translation.len();
    tracing::info!("Updating {} entities 10 times", n_entities);
    for _ in 0..10 {
        tick(&mut world)
    }
}

#[tracing::instrument(skip_all)]
fn tick(world: &mut World) {
    world.tick_number += 1;
    // Звезды не обновляем, сразу к планетам
    for (idx, planet_data) in world.planet.iter().enumerate() {
        if let Some(planet_data) = planet_data {
            let host_pos = world
                .translation
                .get(planet_data.star_id)
                .unwrap()
                .as_ref()
                .unwrap();

            let planet_orbit = world.orbit.get(idx).unwrap().as_ref().unwrap();

            let radius_vector = Vector3::new(planet_orbit.radius, 0.0, 0.0);
            let orbit_turn_angle =
                (2.0 * PI) / (world.tick_number % planet_orbit.period_secs) as f64;

            let orbit_turn_rotation =
                Rotation3::from_axis_angle(&Vector3::z_axis(), orbit_turn_angle);

            let rotated_radius_vector = orbit_turn_rotation.transform_vector(&radius_vector);
            let tilted_rotated_radius_vector =
                planet_orbit.tilt.transform_vector(&rotated_radius_vector);

            let pos = tilted_rotated_radius_vector + host_pos.pos;

            let old_pos = world.translation.get_mut(idx).unwrap().as_mut().unwrap();
            let _ = mem::replace(&mut old_pos.pos, pos);
        }
    }

    // Аналогично - луны после планет
    for (idx, moon_data) in world.moon.iter().enumerate() {
        if let Some(moon_data) = moon_data {
            let host_pos = world
                .translation
                .get(moon_data.planet_id)
                .unwrap()
                .as_ref()
                .unwrap();

            let planet_orbit = world.orbit.get(idx).unwrap().as_ref().unwrap();

            let radius_vector = Vector3::new(planet_orbit.radius, 0.0, 0.0);
            let orbit_turn_angle =
                (2.0 * PI) / (world.tick_number % planet_orbit.period_secs) as f64;

            let orbit_turn_rotation =
                Rotation3::from_axis_angle(&Vector3::z_axis(), orbit_turn_angle);

            let rotated_radius_vector = orbit_turn_rotation.transform_vector(&radius_vector);
            let tilted_rotated_radius_vector =
                planet_orbit.tilt.transform_vector(&rotated_radius_vector);

            let pos = tilted_rotated_radius_vector + host_pos.pos;

            let old_pos = world.translation.get_mut(idx).unwrap().as_mut().unwrap();
            let _ = mem::replace(&mut old_pos.pos, pos);
        }
    }
}

#[tracing::instrument(skip_all)]
fn generate_world() -> World {
    let mut world = World {
        tick_number: 0,
        stars: Vec::new(),
        planet: Vec::new(),
        moon: Vec::new(),
        translation: Vec::new(),
        orbit: Vec::new(),
    };

    for _ in 0..N_STARS {
        let star_id = world.stars.len();

        world.stars.push(Some(Star {}));
        world.planet.push(None);
        world.moon.push(None);
        world.translation.push(Some(Translation {
            pos: Vector3::new(0.0, 0.0, 0.0),
        }));
        world.orbit.push(None);

        for _ in 0..N_PLANETS_PER_STAR {
            let planet_id = world.planet.len();

            let orbit = generate_orbit();
            let pos = generate_pos_of_orbiting(
                world.translation.get(star_id).unwrap().as_ref().unwrap(),
                &orbit,
            );

            world.stars.push(None);
            world.planet.push(Some(Planet { star_id }));
            world.moon.push(None);
            world.translation.push(Some(pos));
            world.orbit.push(Some(orbit));

            for _ in 0..N_MOONS_PER_PLANET {
                let orbit = generate_orbit();
                let pos = generate_pos_of_orbiting(
                    world.translation.get(planet_id).unwrap().as_ref().unwrap(),
                    &orbit,
                );

                world.stars.push(None);
                world.planet.push(None);
                world.moon.push(Some(Moon { planet_id }));
                world.translation.push(Some(pos));
                world.orbit.push(Some(orbit));
            }
        }
    }

    world
}

fn generate_orbit() -> Orbit {
    Orbit {
        tilt: generate_orbit_tilt(),
        radius: fastrand::u64(100_000_000..100_000_000_000) as f64,
        period_secs: fastrand::u64((3 * SECONDS_IN_HOUR)..(36 * SECONDS_IN_MONTH)),
    }
}

fn generate_orbit_tilt() -> Rotation3<f64> {
    let roll = fastrand::isize((ORBIT_AXIS_MAX_TILT.neg())..ORBIT_AXIS_MAX_TILT) as f64;
    let pitch = fastrand::isize((ORBIT_AXIS_MAX_TILT.neg())..ORBIT_AXIS_MAX_TILT) as f64;
    let yaw = fastrand::isize((ORBIT_AXIS_MAX_TILT.neg())..ORBIT_AXIS_MAX_TILT) as f64;

    Rotation3::from_euler_angles(roll, pitch, yaw)
}

fn generate_pos_of_orbiting(host_pos: &Translation, orbit: &Orbit) -> Translation {
    let root_pos = orbit
        .tilt
        .transform_vector(&Vector3::new(orbit.radius, 0.0, 0.0));

    let pos = root_pos + host_pos.pos;

    Translation { pos }
}
