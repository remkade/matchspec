# MatchSpec

A Conda MatchSpec implementation in pure Rust. This allows you to parse a matchspec and validate it against a package to see if it matches.

## Example

The way you instantiate a MatchSpec is by parsing a string into the type:

```rust
use matchspec::MatchSpec;

// Create the MatchSpec by parsing a String or &str
let matchspec: MatchSpec<String> = "main/linux-64::pytorch>1.10.2".parse().unwrap();

// You then have the data accessible inside the MatchSpec struct if you want it
// Package name is the only mandatory field in a matchspec
assert_eq!(&matchspec.package, "pytorch");

// These are optional, so they will be wrapped in an Option
assert_eq!(matchspec.channel, Some("main".to_string()));
assert_eq!(
	matchspec.version,
	Some(matchspec::CompoundSelector::Single {
		selector: matchspec::Selector::GreaterThan,
		version: "1.10.2".to_string(),
	})
);

// You can also check to see if a package name and version match the spec.
// This is a faster function that allows us to bypass some sometimes unnecessary tests like channel or subdir
assert!(matchspec.is_package_version_match(&"pytorch", &"1.11.0"))
```

## Benchmarking

This library contains benchmarks aimed at checking the speed of our implementation against other languages and ensure speed doesn't regress. This is a pure Rust benchmark so you'll need to view it with some skepticism if you want to compare this implementation against others. Benchmark harnesses and the data all need to be identical for a benchmark to really provide value.

### Running the benchmarks

These benchmarks use [Criterion.rs](https://bheisler.github.io/criterion.rs/book/criterion_rs.html) to provide the benchmarking framework. Its pretty easy to run the benchmarks on stable rust:

```bash
cargo bench
```

This will automatically track benchmark timings across runs. If you do this on a laptop or workstation be aware that you may have regressions show up if you have background processes or other things happening. I would recommend always running the benchmarks at a similar level of CPU load. If you want consistent testing its probably best to quit your browser or anything in the background that might be eating CPU or doing IO.
