use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_sandbox::{generate_world, tick};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("10 ticks", |b| {
        b.iter(|| {
            let mut w = black_box(generate_world());
            for _ in 0..10 {
                tick(&mut w)
            }
        })
    });
}

criterion_group! {
  name = benches;
  config = Criterion::default().measurement_time(Duration::from_secs(30));
  targets = criterion_benchmark
}
criterion_main!(benches);
