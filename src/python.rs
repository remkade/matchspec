use crate::matchspec::MatchSpec;
use crate::package_candidate::PackageCandidate;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use pyo3::wrap_pyfunction;
use rayon::prelude::*;

#[pymodule]
fn rust_matchspec(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(match_against_matchspec, m)?)?;
    m.add_function(wrap_pyfunction!(filter_package_list, m)?)?;
    m.add_function(wrap_pyfunction!(parallel_filter_package_list, m)?)?;
    m.add_function(wrap_pyfunction!(parallel_filter_package_list_with_matchspec_list, m)?)?;
    m.add_class::<MatchSpec>()?;
    m.add_class::<PackageCandidate>()?;
    Ok(())
}

/// Conversion function to take a PyList and get a native Vec<PackageCandidate>
fn try_pylist_into_vec_of_package_candidates(
    list: &PyList,
) -> Result<Vec<PackageCandidate>, PyErr> {
    let mut accumulator: Vec<PackageCandidate> = vec![];
    for d in list.into_iter() {
        let dict: &PyDict = d.downcast::<PyDict>()?;
        let pc = PackageCandidate::try_from(dict)?;
        accumulator.push(pc)
    }
    Ok(accumulator)
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

/// Filters a list of package dictionaries against a matchspec in parallel. This doesn't give a
/// noticable speed increase until the list of packages is in the millions
#[pyfunction]
#[pyo3(signature = (matchspec, package_list))]
fn parallel_filter_package_list(
    matchspec: String,
    package_list: &PyList,
) -> Result<Vec<PackageCandidate>, PyErr> {
    let ms = matchspec.parse()?;
    let list = try_pylist_into_vec_of_package_candidates(package_list)?;

    Ok(list
        .par_iter()
        .with_min_len(1000)
        .filter(|pc| pc.is_match(&ms))
        .cloned()
        .collect())
}

/// Helper function to filter a list of PackageCandidate against a MatchSpec
fn filter_package_vec(
    matchspec: &MatchSpec,
    package_list: &[PackageCandidate],
) -> Vec<PackageCandidate> {
    package_list
        .iter()
        .filter(|pc| pc.is_match(matchspec))
        .cloned()
        .collect()
}

/// Takes a list of package dictionaries and filters it based on a list of matchspecs. This runs
/// each matchspec against the package list in paralell, and returns a flat list of package
/// candidates that match any of the given matchspecs.
#[pyfunction]
#[pyo3(signature = (matchspecs, package_list))]
fn parallel_filter_package_list_with_matchspec_list(
    matchspecs: Vec<String>,
    package_list: &PyList,
) -> Result<Vec<PackageCandidate>, PyErr> {
    let mut matchspec_list: Vec<MatchSpec> = Vec::new();
    for maybe_matchspec in matchspecs {
        let ms: MatchSpec = maybe_matchspec.parse()?;
        matchspec_list.push(ms);
    }

    let package_candidate_list: Vec<PackageCandidate> =
        try_pylist_into_vec_of_package_candidates(package_list)?;

    Ok(matchspec_list
        .par_iter()
        .flat_map(|ms| filter_package_vec(ms, &package_candidate_list))
        .collect())
}
