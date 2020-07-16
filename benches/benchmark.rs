use criterion::{criterion_group, criterion_main, Criterion};
use rs_bom::BOM;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("all verses find ephraim", |b| {
        b.iter(|| {
            if let Ok(bom) = BOM::from_default_parser() {
                let ephraim = "ephraim";
                let _ = bom
                    .verses()
                    .filter(|v| v.text.to_lowercase().contains(ephraim))
                    .count();
            }
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
