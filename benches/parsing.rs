use criterion::{black_box, criterion_group, criterion_main, Criterion};
use matchspec::MatchSpec;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("All possible matchers", |b| {
        b.iter(|| {
            // This is a complex but not unlikely matchspec
            black_box("conda-forge/linux-64:NAMESPACE:tensorflow>=1.9.2[license=\"GPL\", subdir=\"linux-64\"]")
                .parse::<MatchSpec<String>>()
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
