pub mod component;
pub mod entity;
pub mod heap;
pub mod system;
pub mod world;


use std::cell::RefCell;
use std::rc::Rc;

use entity::selestial::{
    generate_moons, generate_planets, generate_stars, N_MOONS_PER_PLANET,
    N_PLANETS_PER_STAR, N_STARS,
};


use tracing_subscriber::fmt;
use tracing_subscriber::fmt::format::FmtSpan;
use world::World;

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
    let world = Rc::new(RefCell::new(World::new()));

    generate_stellar(&world);
}

#[tracing::instrument(skip_all)]
pub fn generate_stellar(world: &Rc<RefCell<World>>) {
    println!(
        "Generating {} entities",
        N_STARS
            + (N_STARS * N_PLANETS_PER_STAR)
            + (N_STARS * N_PLANETS_PER_STAR * N_MOONS_PER_PLANET)
    );
    
    let star_ids = generate_stars(Rc::clone(world), N_STARS);
    let planet_ids = generate_planets(Rc::clone(world), star_ids, N_PLANETS_PER_STAR);
    generate_moons(Rc::clone(world), planet_ids, N_MOONS_PER_PLANET);
}
