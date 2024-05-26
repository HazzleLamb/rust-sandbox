use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};
use rust_sandbox::{generate_world, tick};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("selestial tick", |b| {
        b.iter_batched_ref(
            || generate_world(),
            |w| tick(w),
            criterion::BatchSize::LargeInput,
        )
    });
}

criterion_group! {
  name = benches;
  config = Criterion::default().measurement_time(Duration::from_secs(65));
  targets = criterion_benchmark
}
criterion_main!(benches);
