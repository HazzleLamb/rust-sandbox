use std::{f64::consts::PI, ops::Neg};

use nalgebra::{Rotation3, Vector3};

pub struct Star {
    pub pos: Vector3<f64>,
}
pub struct Planet {
    pub star_id: usize,
    pub pos: Vector3<f64>,

    pub tilt: Rotation3<f64>,
    pub radius: f64,
    pub period_secs: u64,
}

pub struct Moon {
    planet_id: usize,
    pos: Vector3<f64>,

    tilt: Rotation3<f64>,
    radius: f64,
    period_secs: u64,
}

pub struct World {
    pub tick_number: u64,
    pub stars: Vec<Option<Star>>,
    pub planet: Vec<Option<Planet>>,
    pub moon: Vec<Option<Moon>>,
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

pub fn tick(w: &mut World) {
    w.tick_number += 1;
    // Звезды не обновляем, сразу к планетам
    for planet_data in w.planet.iter_mut() {
        if let Some(planet_data) = planet_data {
            let host_pos = w.stars[planet_data.star_id].as_ref().unwrap();

            let radius_vector = Vector3::new(planet_data.radius, 0.0, 0.0);
            let orbit_turn_angle = (2.0 * PI) / (w.tick_number % planet_data.period_secs) as f64;

            let orbit_turn_rotation =
                Rotation3::from_axis_angle(&Vector3::z_axis(), orbit_turn_angle);

            let rotated_radius_vector = orbit_turn_rotation.transform_vector(&radius_vector);
            let tilted_rotated_radius_vector =
                planet_data.tilt.transform_vector(&rotated_radius_vector);

            planet_data.pos = tilted_rotated_radius_vector + host_pos.pos;
        }
    }

    // Аналогично - луны после планет
    for moon_data in w.moon.iter_mut() {
        if let Some(moon_data) = moon_data {
            let host_pos = w.planet[moon_data.planet_id].as_ref().unwrap();

            let radius_vector = Vector3::new(moon_data.radius, 0.0, 0.0);
            let orbit_turn_angle = (2.0 * PI) / (w.tick_number % moon_data.period_secs) as f64;

            let orbit_turn_rotation =
                Rotation3::from_axis_angle(&Vector3::z_axis(), orbit_turn_angle);

            let rotated_radius_vector = orbit_turn_rotation.transform_vector(&radius_vector);
            let tilted_rotated_radius_vector =
                moon_data.tilt.transform_vector(&rotated_radius_vector);

            moon_data.pos = tilted_rotated_radius_vector + host_pos.pos;
        }
    }
}

pub fn generate_world() -> World {
    let mut w = World {
        tick_number: 0,
        stars: Vec::new(),
        planet: Vec::new(),
        moon: Vec::new(),
    };

    for _ in 0..N_STARS {
        let star_id = w.stars.len();

        w.stars.push(Some(Star {
            pos: Vector3::new(0.0, 0.0, 0.0),
        }));
        w.planet.push(None);
        w.moon.push(None);

        for _ in 0..N_PLANETS_PER_STAR {
            let planet_id = w.planet.len();

            w.stars.push(None);
            w.planet.push(Some(Planet {
                star_id,
                pos: Vector3::new(0.0, 0.0, 0.0),
                tilt: generate_orbit_tilt(),
                radius: fastrand::u64(100_000_000..100_000_000_000) as f64,
                period_secs: fastrand::u64((3 * SECONDS_IN_HOUR)..(36 * SECONDS_IN_MONTH)),
            }));
            w.moon.push(None);

            for _ in 0..N_MOONS_PER_PLANET {
                w.stars.push(None);
                w.planet.push(None);
                w.moon.push(Some(Moon {
                    planet_id,
                    pos: Vector3::new(0.0, 0.0, 0.0),
                    tilt: generate_orbit_tilt(),
                    radius: fastrand::u64(100_000_000..100_000_000_000) as f64,
                    period_secs: fastrand::u64((3 * SECONDS_IN_HOUR)..(36 * SECONDS_IN_MONTH)),
                }));
            }
        }
    }

    w
}

fn generate_orbit_tilt() -> Rotation3<f64> {
    let roll = fastrand::isize((ORBIT_AXIS_MAX_TILT.neg())..ORBIT_AXIS_MAX_TILT) as f64;
    let pitch = fastrand::isize((ORBIT_AXIS_MAX_TILT.neg())..ORBIT_AXIS_MAX_TILT) as f64;
    let yaw = fastrand::isize((ORBIT_AXIS_MAX_TILT.neg())..ORBIT_AXIS_MAX_TILT) as f64;

    Rotation3::from_euler_angles(roll, pitch, yaw)
}
