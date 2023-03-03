use serde::{Serialize, Deserialize};
use std::fmt::Debug;
use crate::matchspec::*;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct PackageCandidate {
    pub name: String,
    pub version: Option<String>,
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
        ms.is_match(self)
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
            let ms: MatchSpec<String> = "main/linux-64::python>3.10".parse().unwrap();
            let candidate = PackageCandidate::from(payload);
            assert!(ms.is_match(&candidate));

            let ms: MatchSpec<String> = "main/linux-64::python<3.10".parse().unwrap();
            assert!(!candidate.is_match(&ms))
        }
    }
}
