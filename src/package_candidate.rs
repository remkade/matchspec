use crate::matchspec::*;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[pyclass]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct PackageCandidate {
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get, set)]
    pub version: Option<String>,
    #[pyo3(get, set)]
    pub build: Option<String>,
    #[pyo3(get, set)]
    pub build_number: Option<u32>,
    #[serde(default = "Vec::new")]
    #[pyo3(get, set)]
    pub depends: Vec<String>,
    #[pyo3(get, set)]
    pub license: Option<String>,
    #[pyo3(get, set)]
    pub md5: Option<String>,
    #[pyo3(get, set)]
    pub sha256: Option<String>,
    #[pyo3(get, set)]
    pub size: Option<u64>,
    #[pyo3(get, set)]
    pub subdir: Option<String>,
    #[pyo3(get, set)]
    pub timestamp: Option<u64>,
}

// These are safe to assume because Option, String, and u64 are all Send/Sync
unsafe impl Send for PackageCandidate {}
unsafe impl Sync for PackageCandidate {}

impl From<&str> for PackageCandidate {
    fn from(s: &str) -> Self {
        let package_candidate: PackageCandidate = serde_json::from_str(s).unwrap();
        package_candidate
    }
}

#[pymethods]
impl PackageCandidate {
    #[new]
    pub fn new(
        name: String,
        version: Option<String>,
        build: Option<String>,
        build_number: Option<u32>,
        depends: Option<Vec<String>>,
        license: Option<String>,
        md5: Option<String>,
        sha256: Option<String>,
        size: Option<u64>,
        subdir: Option<String>,
        timestamp: Option<u64>,
    ) -> Self {
        PackageCandidate {
            name,
            version,
            build,
            build_number,
            license,
            md5,
            sha256,
            size,
            subdir,
            timestamp,
            depends: depends.unwrap_or_default(),
        }
    }

    pub fn is_match(&self, ms: &MatchSpec) -> bool {
        ms.is_match(self)
    }

    pub fn __repr__(&self) -> String {
        match (&self.name, &self.version, &self.build, &self.build_number) {
            (name, Some(version), Some(build), Some(build_number)) => {
                format!(
                    "PackageCandidate(name={}, version={}, build={}, build_number={})",
                    name, version, build, build_number
                )
            }
            (name, Some(version), None, None) => {
                format!("PackageCandidate(name={}, version={})", name, version)
            }
            _ => format!("PackageCandidate(name={})", self.name),
        }
    }

    #[staticmethod]
    pub fn from_dict(dict: &PyDict) -> Result<Self, PyErr> {
        let any: &PyAny = dict.as_ref();
        let name: String = any.get_item("name")?.to_string();

        let get = |x: &str, dict: &PyDict| -> Option<String> {
            dict.get_item(x).and_then(|i| PyAny::extract(i).ok())
        };

        Ok(PackageCandidate {
            name,
            version: get("version", dict),
            build: get("build", dict),
            build_number: dict
                .get_item("build_number")
                .and_then(|i| PyAny::extract(i).ok()),
            depends: dict
                .get_item("depends")
                .and_then(|i| PyAny::extract::<Vec<String>>(i).ok())
                .unwrap_or_default(),
            license: get("license", dict),
            md5: get("md5", dict),
            sha256: get("sha256", dict),
            size: dict
                .get_item("size")
                .and_then(|i| PyAny::extract(i).ok()),
            subdir: get("subdir", dict),
            timestamp: dict
                .get_item("timestamp")
                .and_then(|i| PyAny::extract(i).ok()),
        })
    }
}

impl TryFrom<&PyDict> for PackageCandidate {
    type Error = PyErr;
    fn try_from(value: &PyDict) -> Result<Self, Self::Error> {
        PackageCandidate::from_dict(value)
    }
}


#[cfg(test)]
mod test {
    #[cfg(test)]
    mod package_candidate {
        use crate::package_candidate::*;

        #[test]
        fn package_candidate_match() {
            let payload = r#"{
                  "build_number": 1,
                  "license": "GPL",
                  "md5": "md5xyz",
                  "name": "python",
                  "sha256": "sha256xyz",
                  "size": 423273,
                  "subdir": "linux-64",
                  "timestamp": 1534356589107,
                  "version": "3.10.4"
                }"#;
            let ms: MatchSpec = "main/linux-64::python>3.10".parse().unwrap();
            let candidate = PackageCandidate::from(payload);
            assert!(ms.is_match(&candidate));

            let ms: MatchSpec = "main/linux-64::python<3.10".parse().unwrap();
            assert!(!candidate.is_match(&ms))
        }
    }
}
