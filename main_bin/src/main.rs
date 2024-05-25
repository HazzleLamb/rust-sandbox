use main_lib::{generate_world, tick};
use tracing_subscriber::fmt::{self, format::FmtSpan};

fn main() {
    fmt::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .with_target(false)
        .with_level(false)
        .init();

    // Используем функции, чтобы было удобно мерять время. Не выкабениваемся
    let mut w = generate_world();

    let n_entities = w.stars.len();
    tracing::info!("Updating {} entities 10 times", n_entities);
    for _ in 0..10 {
        tick(&mut w)
    }
}
