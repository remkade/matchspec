#![doc = include_str ! ("../README.md")]

pub mod matchspec;
mod input_table;
mod parsers;
pub mod package_candidate;

pub use crate::matchspec::*;

#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::wrap_pyfunction;


/// This function matches matchspec string against package name and version
#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(signature = (matchspec, package, version))]
fn match_against_matchspec(matchspec: String, package: String, version: String) -> bool {
    let ms: matchspec::MatchSpec<String> = matchspec.parse().unwrap();
    ms.is_package_version_match(&package, &version).unwrap()
}

#[cfg(feature = "python")]
#[pymodule]
fn rust_matchspec(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(match_against_matchspec, m)?)?;
    Ok(())
}
