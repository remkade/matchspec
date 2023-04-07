# MatchSpec

A Conda MatchSpec implementation in pure Rust. This allows you to parse a matchspec and validate it against a package to see if it matches.

# Python Library

This library exposes a few simple functions:

## `match_against_matchspec()`

Takes a `matchspec` as a `str` and matches it against a `package_name` and `version` (both `str`). Returns a `bool`.

``` python
import rust_matchspec
rust_matchspec.match_against_matchspec('python>=3.0', 'python', '3.10.1') # returns True
```

## `filter_package_list()`

Takes a `list` of `dicts` and returns all the dicts inside that match a given matchspec. The `dicts` must have a `name` key with a `str` value, but all other fields are optional.

```python
import rust_matchspec
list = [{'name': 'tensorflow', 'version': '2.10.0'},
	{'name': 'pytorch', 'version': '2.0.0'},
	{'name': 'pytorch', 'version': '1.11.1'}]

rust_matchspec.filter_package_list('pytorch>1.12', list) # returns [PackageCandidate(name=pytorch)]
```

Possible keys:

| Key          | Expected Type | Required? |
|--------------|---------------|-----------|
| name         | str           | yes       |
| version      | str           |           |
| build        | str           |           |
| build_number | u32           |           |
| depends      | [str]         |           |
| license      | str           |           |
| md5          | str           |           |
| sha256       | str           |           |
| size         | u64           |           |
| subdir       | str           |           |
| timestamp    | u64           |           |

# Rust Library

## Example

The way you instantiate a MatchSpec is by parsing a string into the type:

```rust
use rust_matchspec::{CompoundSelector, MatchSpec, Selector};

// Create the MatchSpec by parsing a String or &str
let matchspec: MatchSpec = "main/linux-64::pytorch>1.10.2".parse().unwrap();

// You then have the data accessible inside the MatchSpec struct if you want it
// Package name is the only mandatory field in a matchspec
assert_eq!(&matchspec.package, "pytorch");

// These are optional, so they will be wrapped in an Option
assert_eq!(matchspec.channel, Some("main".to_string()));
assert_eq!(
	matchspec.version,
	Some(CompoundSelector::Single {
		selector: Selector::GreaterThan,
		version: "1.10.2".to_string(),
	})
);

// You can also check to see if a package name and version match the spec.
// This is a faster function that allows us to bypass some sometimes unnecessary tests like channel or subdir
assert!(matchspec.is_package_version_match(&"pytorch", &"1.11.0"))
```

## Benchmarking

This library contains benchmarks aimed at checking the speed of our implementation against other languages and ensure speed doesn't regress. These are contrived benchmarks to test raw speed, so take them (and all benchmarks) with a bit of skepticism. Benchmark harnesses and the data all need to be identical for a benchmark to really provide value.


### Python

The Python benchmarks use [pytest-benchmark](https://pytest-benchmark.readthedocs.io/en/stable/).

Steps to run the benchmarks:

```bash
# Setup the conda env
conda env create -f ./environment.yml
conda activate rust_matchspec

# Build an optimized wheel
maturin build --release

# install it
pip install ./target/wheels/rust_matchspec*.whl

# Finally, run the benchmark
pytest
```

### Rust

The Rust benchmarks use [Criterion.rs](https://bheisler.github.io/criterion.rs/book/criterion_rs.html) to provide the benchmarking framework. Its pretty easy to run the benchmarks on stable rust:

```bash
cargo bench 

# Or if you're on mac and get errors with Invalid Symbols:
cargo bench --no-default-features
```
This will automatically track benchmark timings across runs. If you do this on a laptop or workstation be aware that you may have regressions show up if you have background processes or other things happening. I would recommend always running the benchmarks at a similar level of CPU load. If you want consistent testing its probably best to quit your browser or anything in the background that might be eating CPU or doing IO.
