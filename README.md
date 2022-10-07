# MatchSpec

A Conda MatchSpec implementation in pure Rust. This allows you to parse a matchspec and validate it against a package to see if it matches.

## Example

The way you instantiate a MatchSpec is by parsing a string into the type:

```rust
use matchspec::MatchSpec;

let matchspec: MatchSpec = "main/linux-64::pytorch>1.10.2";

assert_eq!(matchspec.name, "pytorch".to_string());
assert_eq!(matchspec.selector, matchspec::Selector::GreaterThan);
assert_eq!(matchspec.version, "1.10.2".to_string());
```
