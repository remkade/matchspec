use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use crate::matchspec::*;

/// This function matches matchspec string against package name and version
#[pyfunction]
#[pyo3(signature = (matchspec, package, version))]
fn match_against_matchspec(matchspec: String, package: String, version: String) -> bool {
    let ms: MatchSpec<String> = matchspec.parse().unwrap();
    ms.is_package_version_match(&package, &version).unwrap()
}

#[pymodule]
fn rust_matchspec(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(match_against_matchspec, m)?)?;
    Ok(())
}
