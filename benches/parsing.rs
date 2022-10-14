use criterion::{black_box, criterion_group, criterion_main, Criterion};
use matchspec::MatchSpec;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Package name only", |b| {
        b.iter(|| {
            // This is a complex but not unlikely matchspec
            black_box("tzdata")
                .parse::<MatchSpec<String>>()
        })
    });
    c.bench_function("Package name and version", |b| {
        b.iter(|| {
            // This is a complex but not unlikely matchspec
            black_box("openssl>1.1.1g")
                .parse::<MatchSpec<String>>()
        })
    });
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
