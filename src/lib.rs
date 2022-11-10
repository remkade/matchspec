#![doc = include_str!("../README.md")]
pub mod matchspec;
mod parsers;
pub use crate::matchspec::*;

use pyo3::prelude::*;
use pyo3::wrap_pyfunction;


#[pyfunction]
fn match_against_matchspec(matchspec: String, package: String, version: String) -> bool {
  let ms: matchspec::MatchSpec<String> = matchspec.parse().unwrap();
  ms.is_package_version_match(&package, &version)
}

#[pymodule]
fn rust_matchspec(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(match_against_matchspec, m)?)?;
    Ok(())
}
