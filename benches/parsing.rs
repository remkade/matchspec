use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_matchspec::matchspec::MatchSpec;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Package name only", |b| {
        b.iter(|| {
            // This is a complex but not unlikely matchspec
            black_box("tzdata").parse::<MatchSpec>()
        })
    });
    c.bench_function("Package name and version", |b| {
        b.iter(|| {
            // This is a complex but not unlikely matchspec
            black_box("openssl>1.1.1g").parse::<MatchSpec>()
        })
    });
    c.bench_function("All possible matchers", |b| {
        b.iter(|| {
            // This is a complex but not unlikely matchspec
            black_box("conda-forge/linux-64:NAMESPACE:tensorflow>=1.9.2[license=\"GPL\", subdir=\"linux-64\"]")
                .parse::<MatchSpec>()
        })
    });

    c.bench_function("Repodata depends", |b| {
        let depends_file = format!(
            "{}/test_data/linux_64-depends.txt",
            env!("CARGO_MANIFEST_DIR")
        );
        let repodata_depends_buffer =
            BufReader::new(File::open(depends_file).expect("opening repodata depends file"));
        let depends: Vec<String> = repodata_depends_buffer
            .lines()
            .map(|l| l.unwrap())
            .collect();
        b.iter(|| {
            for d in &depends {
                d.parse::<MatchSpec>().unwrap();
            }
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
