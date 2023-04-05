use crate::matchspec::MatchSpec;
use crate::package_candidate::PackageCandidate;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use pyo3::{create_exception, exceptions::PyException, wrap_pyfunction};

create_exception!(rust_matchspec, MatchSpecParsingError, PyException);

#[pymodule]
fn rust_matchspec(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(match_against_matchspec, m)?)?;
    m.add_function(wrap_pyfunction!(filter_package_list, m)?)?;
    m.add_class::<MatchSpec>()?;
    m.add_class::<PackageCandidate>()?;
    Ok(())
}

/// This function matches matchspec string against package name and version
#[pyfunction]
#[pyo3(signature = (matchspec, package, version))]
fn match_against_matchspec(matchspec: String, package: String, version: String) -> bool {
    let ms: MatchSpec = matchspec.parse().unwrap();
    ms.is_package_version_match(&package, &version)
}

/// Take a list of dicts returning a filtered list that matches the given matchspec.
#[pyfunction]
#[pyo3(signature = (matchspec, package_list))]
fn filter_package_list(
    py: Python,
    matchspec: String,
    package_list: &PyList,
) -> Result<Py<PyList>, PyErr> {
    // This will be used later to abort if the list given doesn't have a proper dict
    let mut err = Ok(());
    let ms: MatchSpec = matchspec.parse().unwrap();

    // Loop through the pylist and create a Vec<PackageCandidate>
    let filtered: Vec<PackageCandidate> = package_list
        .iter()
        .map(|i| i.downcast::<PyDict>())
        // If we encounter any invalid dicts we'll assign it to the accumalator and fail
        .scan(&mut err, |err, res| match res {
            Ok(o) => Some(o),
            Err(e) => {
                **err = Err(e);
                None
            }
        })
        .flat_map(PackageCandidate::from_dict)
        .filter(|pc| pc.is_match(&ms))
        .collect();

    let pylist = PyList::new(py, filtered.iter().map(|pc| pc.clone().into_py(py)));

    // Looks weird, but lets raise the error
    err?;

    Ok(pylist.into())
}
