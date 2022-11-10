use serde::{Serialize, Deserialize};
use std::fmt::Debug;
use crate::matchspec::*;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct PackageCandidate {
    pub name: String,
    pub build: Option<String>,
    pub build_number: Option<u32>,
    #[serde(default = "Vec::new")]
    pub depends: Vec<String>,
    pub license: Option<String>,
    pub md5: Option<String>,
    pub sha256: Option<String>,
    pub size: Option<u64>,
    pub subdir: Option<String>,
    pub timestamp: Option<u64>,
    pub version: Option<String>,
}

impl<S> From<S> for PackageCandidate
    where
        S: AsRef<str>,
{
    fn from(s: S) -> Self {
        let package_candidate: PackageCandidate = serde_json::from_str(s.as_ref()).unwrap();
        package_candidate
    }
}

impl PackageCandidate {
    pub fn is_match(&self, ms: &MatchSpec<String>) -> bool {
        ms.is_match(&self)
    }
}

#[cfg(test)]
mod test {
    #[cfg(test)]
    mod package_candidate {
        use crate::matchspec::*;
        use crate::package_candidate::*;

        #[test]
        fn package_candidate_mathing() {
            let payload = r#"{
                      "build": "py35h14c3975_1",
                      "name": "python",
                      "version": "3.10.4"
                    }"#;

            let ms: MatchSpec<String> = "python>3.10".parse().unwrap();
            let candidate = PackageCandidate::from(payload);
            assert!(ms.is_match(&candidate));

            let ms_less: MatchSpec<String> = "python<3.10".parse().unwrap();
            assert!(!candidate.is_match(&ms_less))
        }
    }
}